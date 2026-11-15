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
    static void sourceChanged() {
        if (!executing) return;
        var mc=Minecraft.getInstance(); cancel(mc); TickInput.release(mc);
        Bot.reply(mc,"Native legacy playback stopped on a source event; continuation repair belongs to F.");
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
        if (executing) {
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
            long received = NativeChannel.clock()[0];
            StateBits.Observation observation = PlayerSnapshot.observed(mc);
            HopCapture.hop("request-observation-bits",version,issue,received,observation.bits());
            HopCapture.hop("request-observation-known",version,issue,received,observation.known());
            HopCapture.hop("request-observation-provenance",version,issue,received,observation.provenance());
            long[] meta = PlayerSnapshot.context(mc,received);
            HopCapture.hop("request-context",version,issue,received,meta);
            BlockPos goal = pending; pending = null;
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
        if (answer.length != 9+n+count*3 || !Arrays.equals(Arrays.copyOfRange(answer,9,9+n),now.bits())) {
            Bot.reply(mc,"Native plan refused: observed start changed while planning; event re-anchor belongs to F."); return;
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
