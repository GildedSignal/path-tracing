package pathtracing;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.block.state.BlockState;

/** New-section palettes and block edits, from actual packet application only. */
public final class ResidentSource {
    static long session, revision, lawRevision;
    public static void resetWorld() { join(Minecraft.getInstance()); }
    private static final Map<BlockPos,long[]> sections = new HashMap<>();
    static void join(Minecraft mc) {
        ResidentRunner.cancel(mc);
        session++; revision = 1; lawRevision = 1; sections.clear();
        if (NativeChannel.loadFailure() != null) { Bot.reply(mc,"Native channel refused: " + NativeChannel.loadFailure()); return; }
        // Natural regeneration is not sent to a remote client; 2 means Unknown.
        sentDifficulty = mc.level.getDifficulty().getId();
        send(new long[]{0,session,revision,sentDifficulty,2});
    }
    private static long sentDifficulty;
    /** The server's difficulty packet comes after the join: publish it when it differs from what was sent. */
    static void difficulty(Minecraft mc) {
        if (session == 0 || mc.level == null || mc.level.getDifficulty().getId() == sentDifficulty) return;
        sentDifficulty = mc.level.getDifficulty().getId();
        send(new long[]{4,session,++revision,sentDifficulty});
    }
    private static void send(long[] data) { send(data, true); }
    private static void send(long[] data, boolean changed) {
        long[] answer = NativeChannel.PublishSourceDelta(data);
        if (answer[0] == 3) throw new IllegalStateException("Source publication refused");
        boolean lawChanged = changed && (answer.length < 2 || answer[1] != 0);
        if (lawChanged) {
            lawRevision++;
            ResidentRunner.sourceChanged(data);
        }
    }
    public static void chunk(int x, int z) {
        var mc = Minecraft.getInstance();
        if (mc.level == null || NativeChannel.loadFailure() != null || session == 0) return;
        var chunk = mc.level.getChunk(x,z);
        var all = chunk.getSections();
        for (int s = 0; s < all.length; s++) {
            BlockPos address = new BlockPos(x,mc.level.getMinSectionY()+s,z);
            // A section holds a few distinct states (block states are shared instances): name each once.
            java.util.IdentityHashMap<BlockState,Integer> palette = new java.util.IdentityHashMap<>();
            ArrayList<Long> header = new ArrayList<>();
            ArrayList<BlockState> order = new ArrayList<>();
            long[] indices = new long[4096];
            for (int y=0;y<16;y++) for (int bz=0;bz<16;bz++) for (int bx=0;bx<16;bx++) {
                BlockState state = all[s].getBlockState(bx,y,bz);
                Integer index = palette.get(state);
                if (index == null) { index = order.size(); palette.put(state,index); order.add(state); }
                indices[(y*16+bz)*16+bx] = index;
            }
            header.add(1L); header.add(session); header.add(0L);
            header.add((long)x); header.add((long)address.getY()); header.add((long)z); header.add((long)order.size());
            for (BlockState state : order) text(header,canonical(state));
            long[] words = new long[header.size() + 4096];
            for (int i = 0; i < header.size(); i++) words[i] = header.get(i);
            System.arraycopy(indices,0,words,header.size(),4096);
            long[] previous = sections.get(address);
            if (previous != null && previous.length > 2) words[2] = previous[2];
            if (java.util.Arrays.equals(previous,words)) continue;
            words[2] = ++revision; sections.put(address,words); send(words);
        }
    }
    /** `before` is what the client held before the packet (null when unknown): an equal state is no change. */
    public static void block(BlockPos p, BlockState state, BlockState before) {
        if (NativeChannel.loadFailure() != null || session == 0) return;
        BlockPos address = new BlockPos(p.getX()>>4,p.getY()>>4,p.getZ()>>4);
        if (!sections.containsKey(address)) return; // Not client-loaded: never grant a lease.
        sections.remove(address); // A replacement section packet must compare against fresh contents.
        // Keep the loaded address; contents are owned by Rust after the first section.
        sections.put(address,new long[0]);
        ArrayList<Long> data = new ArrayList<>();
        for (long value : new long[]{2,session,++revision,p.getX(),p.getY(),p.getZ()}) data.add(value);
        text(data,canonical(state));
        if (before != null) text(data,canonical(before));
        send(data.stream().mapToLong(Long::longValue).toArray(), !state.equals(before));
    }
    public static void unload(int x,int z) {
        if (NativeChannel.loadFailure() != null) return;
        for (BlockPos p : java.util.List.copyOf(sections.keySet())) if (p.getX()==x && p.getZ()==z) {
            sections.remove(p); send(new long[]{3,session,++revision,x,p.getY(),z});
        }
    }
    static String canonical(BlockState state) {
        String properties = state.getValues().map(Object::toString).sorted().collect(java.util.stream.Collectors.joining(","));
        return BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString() + (properties.isEmpty()?"":"["+properties+"]");
    }
    private static void text(ArrayList<Long> data,String text) {
        byte[] bytes = text.getBytes(StandardCharsets.UTF_8); data.add((long)bytes.length);
        for (byte b : bytes) data.add(Byte.toUnsignedLong(b));
    }
    private ResidentSource() { }
}
