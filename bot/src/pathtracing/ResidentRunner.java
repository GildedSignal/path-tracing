package pathtracing;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;

/** A-only legacy_sentence_adapter consumer; F owns the eventual word runner. */
final class ResidentRunner {
    private static final long[] EMPTY = new long[0];
    private static long version, issue;
    private static BlockPos pending;
    private static boolean waiting;
    private static boolean executing;
    static void playbackEnded() { executing=false; actions=List.of(); }
    // The plan's corridor in x and z, as the planner reads it: a block edit inside it can change the plan; new chunks cannot.
    private static int[] corridor = new int[4];
    // The plan's own edits: their echoes from the server are expected, every other edit stops the flight.
    private static List<KitAction> actions = List.of();
    private static int stalls;
    static void sourceChanged(long[] data) {
        if (!executing || data[0] != 2 || data[3] < corridor[0] || data[3] > corridor[2] || data[5] < corridor[1] || data[5] > corridor[3]) return;
        if (KitAction.ours(actions, Recorder.planTick() - 1, (int) data[3], (int) data[4], (int) data[5])) return;
        var mc=Minecraft.getInstance(); cancel(mc); TickInput.release(mc);
        Bot.reply(mc,"Stopped: the block at " + data[3] + " " + data[4] + " " + data[5] + " changed inside the plan's corridor.");
    }
    /** The kit actions due at plan tick `t`; false holds the tick (a break still mining). */
    static boolean act(Minecraft mc, int t) {
        for (KitAction a : actions) {
            if (t < a.tick() || t >= a.tick() + a.hold()) continue;
            if (a.perform(mc, t - a.tick())) { stalls = 0; continue; }
            if (++stalls > 8) {
                cancel(mc); TickInput.release(mc);
                Bot.reply(mc, "Stopped: the block at " + a.x() + " " + a.y() + " " + a.z() + " did not break in time.");
            }
            return false;
        }
        return true;
    }
    static void event(String kind) {
        var mc = Minecraft.getInstance();
        if (mc.player == null || mc.level == null || NativeChannel.loadFailure() != null || ResidentSource.session == 0) return;
        StateBits.Observation observation = PlayerSnapshot.observed(mc);
        long at = NativeChannel.clock()[0];
        long[] reply = NativeChannel.ObserveEvent(new long[]{1,version,at,ResidentSource.session,ResidentSource.revision},observation.bits(),observation.known(),observation.provenance());
        HopCapture.hop("observe-"+kind,version,issue,at,reply);
        HopCapture.hop("observation-bits-"+kind,version,issue,at,observation.bits());
        HopCapture.hop("observation-known-"+kind,version,issue,at,observation.known());
        HopCapture.hop("observation-provenance-"+kind,version,issue,at,observation.provenance());
        // Only events that move the body stop playback; data echoes (entity data, attributes, health and food) are recorded above.
        if (executing && java.util.Set.of("explosion","motion","position","rotation").contains(kind)) {
            cancel(mc); TickInput.release(mc);
            Bot.reply(mc,"Native legacy playback stopped on observed "+kind+"; event comparison belongs to F.");
        }
    }
    static void request(Minecraft mc, BlockPos goal, long issuance) {
        cancel(mc); issue = issuance; pending = goal.immutable();
        if (NativeChannel.loadFailure() != null) {
            pending = null; Bot.reply(mc,"Native channel refused: " + NativeChannel.loadFailure());
        }
    }
    static void cancel(Minecraft mc) {
        version++; pending = null; waiting = false; executing = false; actions = List.of();
        if (NativeChannel.loadFailure() == null) NativeChannel.Cancel(version);
        Recorder.cancelPlan(mc);
    }
    static void tick(Minecraft mc) {
        if (mc.player == null || mc.level == null || NativeChannel.loadFailure() != null) return;
        if (pending != null) {
            ResidentSource.difficulty(mc);
            long received = NativeChannel.clock()[0];
            StateBits.Observation observation = PlayerSnapshot.observed(mc);
            HopCapture.hop("request-observation-bits",version,issue,received,observation.bits());
            HopCapture.hop("request-observation-known",version,issue,received,observation.known());
            HopCapture.hop("request-observation-provenance",version,issue,received,observation.provenance());
            long[] meta = PlayerSnapshot.context(mc,received);
            HopCapture.hop("request-context",version,issue,received,meta);
            BlockPos goal = pending; pending = null;
            BlockPos at = mc.player.blockPosition();
            corridor = new int[]{Math.min(at.getX(),goal.getX())-2,Math.min(at.getZ(),goal.getZ())-2,Math.max(at.getX(),goal.getX())+2,Math.max(at.getZ(),goal.getZ())+2};
            long[] answer = NativeChannel.RequestGoal(observation.bits(),observation.known(),observation.provenance(),meta,
                new long[]{goal.getX(),goal.getY()-1,goal.getZ()},issue,version);
            HopCapture.hop("mod-request",version,issue,received,answer);
            waiting = answer[0] == 0;
            if (!waiting) refuse(mc,answer);
        }
        if (!waiting) return;
        long[] answer = NativeChannel.ObserveEvent(new long[]{0,version},EMPTY,EMPTY,EMPTY);
        if (answer[0] == 0) return;
        if (answer[1] != version && answer[1] != 0) return;
        waiting = false;
        HopCapture.hop("rust-result-mod",version,issue,NativeChannel.clock()[0],answer);
        // 1 = WORD_PLAN; 4 = KIT_PLAN, the same with [a, a × 10 action words] between the inputs and the hull.
        if (answer[0] != 1 && answer[0] != 4) { refuse(mc,answer); return; }
        if (answer.length < 9 || answer[5] != ResidentSource.revision) { Bot.reply(mc,"Native plan refused: source changed."); return; }
        int count = Math.toIntExact(answer[7]), n = Math.toIntExact(answer[8]);
        int inputsEnd = 9+n+count*3, kits = answer[0] == 4 && answer.length > inputsEnd ? Math.toIntExact(answer[inputsEnd]) : 0;
        int hullBase = answer[0] == 4 ? inputsEnd+1+kits*KitAction.WORDS : inputsEnd;
        StateBits.Observation now = PlayerSnapshot.observed(mc);
        // Word 0 is the tick counter, which moves while the plan is thought; every other word must still hold.
        int changed = answer.length != hullBase+4 || now.bits().length != n ? 0 : -1;
        for (int i = 1; changed < 0 && i < n; i++) if (answer[9+i] != now.bits()[i]) changed = i;
        if (changed >= 0) {
            Bot.reply(mc,"Native plan refused: observed start changed while planning at word " + changed + "."); return;
        }
        // T = 0: the body already touches the goal.
        if (count == 0) { Bot.reply(mc,"Already there."); return; }
        corridor = new int[]{(int)answer[hullBase],(int)answer[hullBase+1],(int)answer[hullBase+2],(int)answer[hullBase+3]};
        actions = KitAction.read(answer,inputsEnd+1,kits); stalls = 0;
        ArrayList<TickInput> inputs = new ArrayList<>(count);
        for (int i=9+n;i<inputsEnd;i+=3) {
            long keys=answer[i];
            inputs.add(new TickInput((keys&1)!=0,(keys&2)!=0,(keys&4)!=0,(keys&8)!=0,(keys&16)!=0,(keys&32)!=0,(keys&64)!=0,
                Float.intBitsToFloat((int)answer[i+1]),Float.intBitsToFloat((int)answer[i+2])));
        }
        Recorder.legacy_sentence_adapter(mc,inputs);
        executing = true;
    }
    private static void refuse(Minecraft mc,long[] answer) {
        if (answer[0] == 2) Bot.reply(mc,"Native planner: Hard, no certified sentence.");
        else {
            StringBuilder reason = new StringBuilder(); for (int i=4;i<answer.length;i++) reason.append((char)answer[i]);
            Bot.reply(mc,"Native channel refused: " + reason);
        }
    }
    private ResidentRunner() { }
}
