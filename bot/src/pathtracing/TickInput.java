package pathtracing;

import com.google.gson.JsonObject;
import net.minecraft.client.Minecraft;

record TickInput(boolean forward, boolean back, boolean left, boolean right, boolean jump, boolean sneak,
                 boolean sprint, float yaw, float pitch) {
    static JsonObject settings(Minecraft mc) {
        JsonObject out = new JsonObject();
        out.addProperty("toggleSprint", mc.options.toggleSprint().get());
        out.addProperty("toggleCrouch", mc.options.toggleCrouch().get());
        out.addProperty("autoJump", mc.options.autoJump().get());
        out.addProperty("sprintWindow", mc.options.sprintWindow().get());
        return out;
    }

    static JsonObject holdSettings(Minecraft mc) {
        JsonObject out = settings(mc);
        if (mc.options.toggleSprint().get() || mc.options.toggleCrouch().get() || mc.options.autoJump().get()
            || mc.options.sprintWindow().get() != 0) {
            throw new IllegalStateException("Requires hold-style input: " + out);
        }
        return out;
    }

    static TickInput read(JsonObject in) {
        for (String key : new String[] {"forward", "back", "left", "right", "jump", "sneak", "sprint"}) {
            if (!in.has(key) || !in.get(key).isJsonPrimitive() || !in.getAsJsonPrimitive(key).isBoolean()) {
                throw new IllegalArgumentException("Expected boolean " + key);
            }
        }
        for (String key : new String[] {"yaw", "pitch"}) {
            if (!in.has(key) || !in.get(key).isJsonPrimitive() || !in.getAsJsonPrimitive(key).isNumber()
                || !Float.isFinite(in.get(key).getAsFloat())) throw new IllegalArgumentException("Expected finite " + key);
        }
        return new TickInput(in.get("forward").getAsBoolean(), in.get("back").getAsBoolean(),
            in.get("left").getAsBoolean(), in.get("right").getAsBoolean(), in.get("jump").getAsBoolean(),
            in.get("sneak").getAsBoolean(), in.get("sprint").getAsBoolean(),
            in.get("yaw").getAsFloat(), in.get("pitch").getAsFloat());
    }

    static TickInput capture(Minecraft mc) {
        return new TickInput(mc.options.keyUp.isDown(), mc.options.keyDown.isDown(), mc.options.keyLeft.isDown(),
            mc.options.keyRight.isDown(), mc.options.keyJump.isDown(), mc.options.keyShift.isDown(),
            mc.options.keySprint.isDown(), mc.player.getYRot(), mc.player.getXRot());
    }

    void apply(Minecraft mc) {
        mc.options.keyUp.setDown(forward);
        mc.options.keyDown.setDown(back);
        mc.options.keyLeft.setDown(left);
        mc.options.keyRight.setDown(right);
        mc.options.keyJump.setDown(jump);
        mc.options.keyShift.setDown(sneak);
        mc.options.keySprint.setDown(sprint);
        mc.player.setYRot(yaw);
        mc.player.setXRot(pitch);
    }

    static void release(Minecraft mc) {
        mc.options.keyUp.setDown(false);
        mc.options.keyDown.setDown(false);
        mc.options.keyLeft.setDown(false);
        mc.options.keyRight.setDown(false);
        mc.options.keyJump.setDown(false);
        mc.options.keyShift.setDown(false);
        mc.options.keySprint.setDown(false);
    }

    JsonObject json() {
        JsonObject out = new JsonObject();
        out.addProperty("forward", forward);
        out.addProperty("back", back);
        out.addProperty("left", left);
        out.addProperty("right", right);
        out.addProperty("jump", jump);
        out.addProperty("sneak", sneak);
        out.addProperty("sprint", sprint);
        TraceJson.number(out, "yaw", yaw);
        TraceJson.number(out, "pitch", pitch);
        return out;
    }
}
