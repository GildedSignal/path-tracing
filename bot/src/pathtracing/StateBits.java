package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;

/** Exact law/state.rs word order. Unknown placeholders always carry mask=0. */
final class StateBits {
    record Observation(long[] bits, long[] known, long[] provenance) { }
    private final ArrayList<Long> bits = new ArrayList<>(), known = new ArrayList<>(), provenance = new ArrayList<>();
    private void word(long value, boolean present, int owner) {
        bits.add(value); known.add(present ? 1L : 0L); provenance.add(present ? (long) owner : 0L);
    }
    private void integer(JsonObject in, String key) { word(in.has(key) ? in.get(key).getAsLong() : 0, in.has(key), 1); }
    private void bool(JsonObject in, String key) { word(in.has(key) && in.get(key).getAsBoolean() ? 1 : 0, in.has(key), 1); }
    private void floating(JsonObject in, String key, boolean wide) {
        boolean present = in.has(key + "Bits");
        long value = present ? Long.parseUnsignedLong(in.get(key + "Bits").getAsString(),16) : 0;
        if (!wide && (value & ~0xffffffffL) != 0) throw new IllegalArgumentException("f32 bits " + key);
        word(value,present,1);
    }
    private void vector(JsonObject in, String key) {
        JsonObject v = in.has(key) ? in.getAsJsonObject(key) : new JsonObject();
        for (String axis : new String[]{"x","y","z"}) floating(v,axis,true);
    }
    static Observation encode(JsonObject in) {
        StateBits out = new StateBits();
        out.integer(in,"tickCount");
        JsonObject input = in.has("inputs") ? in.getAsJsonObject("inputs") : new JsonObject();
        out.floating(input,"yaw",false); out.floating(input,"pitch",false);
        out.floating(in,"yRot",false); out.floating(in,"xRot",false);
        long pose = in.has("pose") ? switch (in.get("pose").getAsString()) {
            case "STANDING" -> 0; case "CROUCHING" -> 1; case "SWIMMING" -> 2;
            default -> throw new IllegalArgumentException("Unsupported observed pose");
        } : 0;
        out.word(pose,in.has("pose"),1);
        out.integer(in,"hurtRemainingTicks"); out.floating(in,"lastRawHit",false);
        for (String key : new String[]{"foodLevel","foodTickTimer","remainingFireTicks","noJumpDelay","ticksFrozen"}) out.integer(in,key);
        for (String key : new String[]{"forward","back","left","right","jump","sneak","sprint"}) out.bool(input,key);
        for (String key : new String[]{"onGround","horizontalCollision","verticalCollision","verticalCollisionBelow","minorHorizontalCollision","isSprinting","sprintSpeedModifier","noPhysics","isShiftKeyDown","isInWater","isInLava","onClimbable","onGroundNoBlocks","crouching","swimming","underwater","eyeInWater","wasEyeInWater","inPowderSnow","wasInPowderSnow","leatherBoots","fallFlying","riding","boat","effects","flying","isUsingItem"}) out.bool(in,key);
        for (String key : new String[]{"health","saturationLevel","exhaustion","speed","stepHeight","frictionModifier","airDragModifier","waterMovementEfficiency","movementEfficiency","sneakingSpeed"}) out.floating(in,key,false);
        for (String key : new String[]{"useItemSpeedMultiplier","useItemCanSprint"}) {
            boolean present = in.has(key) || in.has(key + "Bits");
            boolean absent = in.has(key + "Absent") && in.get(key + "Absent").getAsBoolean();
            out.word(present ? 1 : 0,present || absent,1);
            if (present) { if (key.endsWith("Sprint")) out.bool(in,key); else out.floating(in,key,false); }
        }
        for (String key : new String[]{"fallDistance","waterHeight","lavaHeight","movementSpeedBase","jumpStrength","gravity","safeFallDistance","fallDamageMultiplier","bounciness"}) out.floating(in,key,true);
        for (String key : new String[]{"position","velocity","stuckSpeedMultiplier"}) out.vector(in,key);
        boolean moved = in.has("movementThisTick");
        boolean projected = in.has("movementThisTickProjected");
        out.word(moved ? 1 : 0,moved || projected || in.has("movementThisTickAbsent"),projected ? 3 : 1);
        if (moved) out.vector(in,"movementThisTick");
        boolean support = in.has("mainSupportingBlock");
        out.word(support ? 1 : 0,support || in.has("mainSupportingBlockKnown"),1);
        if (support) for (var coordinate : in.getAsJsonArray("mainSupportingBlock")) out.word(coordinate.getAsInt(),true,1);
        boolean modifiers = in.has("speedModifiers");
        JsonArray rows = modifiers ? in.getAsJsonArray("speedModifiers") : new JsonArray();
        out.word(rows.size(),modifiers,1);
        for (var value : rows) {
            var row = value.getAsJsonArray();
            byte[] id = row.get(0).getAsString().getBytes(StandardCharsets.UTF_8);
            out.word(id.length,true,1); for (byte b : id) out.word(Byte.toUnsignedLong(b),true,1);
            out.word(Double.doubleToRawLongBits(row.get(1).getAsDouble()),true,1); out.word(row.get(2).getAsInt(),true,1);
        }
        return new Observation(array(out.bits),array(out.known),array(out.provenance));
    }
    private static long[] array(ArrayList<Long> values) { return values.stream().mapToLong(Long::longValue).toArray(); }
    private StateBits() { }
}
