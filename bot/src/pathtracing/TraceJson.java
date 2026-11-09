package pathtracing;

import com.google.gson.JsonObject;
import com.google.gson.JsonArray;
import java.util.Locale;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.phys.Vec3;

/** Decimal values for inspection, raw IEEE bits for the movement judge. */
final class TraceJson {
    static boolean sprintSpeedModifier(AttributeInstance attribute) {
        return attribute.getModifiers().stream().anyMatch(modifier -> modifier.id().toString().equals("minecraft:sprinting"));
    }

    static void number(JsonObject out, String name, double value) {
        out.addProperty(name, value);
        out.addProperty(name + "Bits", String.format(Locale.ROOT, "%016x", Double.doubleToRawLongBits(value)));
    }

    static void number(JsonObject out, String name, float value) {
        out.addProperty(name, value);
        out.addProperty(name + "Bits", String.format(Locale.ROOT, "%08x", Float.floatToRawIntBits(value)));
    }

    static JsonObject vector(Vec3 value) {
        JsonObject out = new JsonObject();
        number(out, "x", value.x);
        number(out, "y", value.y);
        number(out, "z", value.z);
        return out;
    }

    static JsonObject vectorBits(Vec3 value) {
        JsonObject out = vector(value);
        out.remove("x");
        out.remove("y");
        out.remove("z");
        return out;
    }

    static JsonArray speedModifiers(AttributeInstance attribute) {
        JsonArray out = new JsonArray();
        for (var modifier : attribute.getModifiers()) {
            if (modifier.id().toString().equals("minecraft:sprinting")) continue;
            JsonArray row = new JsonArray();
            row.add(modifier.id().toString());
            row.add(modifier.amount());
            row.add(modifier.operation().id());
            out.add(row);
        }
        return out;
    }
}
