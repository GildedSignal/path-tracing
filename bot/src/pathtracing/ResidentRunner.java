package pathtracing;

import java.util.ArrayList;
import java.util.Arrays;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;

/** A-only legacy_sentence_adapter consumer; F owns the eventual word runner. */
final class ResidentRunner {
    private static final long[] EMPTY = new long[0];
    private static long version, issue;
    private static BlockPos pending;
    private static boolean waiting;
    private static boolean executing;
    static void playbackEnded() { executing=false; }
    // The plan's corridor in x and z, as the planner reads it: a block edit inside it can change the plan; new chunks cannot.
    private static int[] corridor = new int[4];
    static void sourceChanged(long[] data) {
        if (!executing || data[0] != 2 || data[3] < corridor[0] || data[3] > corridor[2] || data[5] < corridor[1] || data[5] > corridor[3]) return;
        var mc=Minecraft.getInstance(); cancel(mc); TickInput.release(mc);
        Bot.reply(mc,"Stopped: the block at " + data[3] + " " + data[4] + " " + data[5] + " changed inside the plan's corridor.");
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
        version++; pending = null; waiting = false; executing = false;
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
            corridor = new int[]{Math.min(at.getX(),goal.getX())-8,Math.min(at.getZ(),goal.getZ())-8,Math.max(at.getX(),goal.getX())+8,Math.max(at.getZ(),goal.getZ())+8};
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
        if (answer[0] != 1) { refuse(mc,answer); return; }
        if (answer.length < 9 || answer[5] != ResidentSource.revision) { Bot.reply(mc,"Native plan refused: source changed."); return; }
        int count = Math.toIntExact(answer[7]), n = Math.toIntExact(answer[8]);
        StateBits.Observation now = PlayerSnapshot.observed(mc);
        // Word 0 is the tick counter, which moves while the plan is thought; every other word must still hold.
        int changed = answer.length != 9+n+count*3 || now.bits().length != n ? 0 : -1;
        for (int i = 1; changed < 0 && i < n; i++) if (answer[9+i] != now.bits()[i]) changed = i;
        if (changed >= 0) {
            Bot.reply(mc,"Native plan refused: observed start changed while planning at word " + changed + "."); return;
        }
        ArrayList<TickInput> inputs = new ArrayList<>(count);
        for (int i=9+n;i<answer.length;i+=3) {
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
