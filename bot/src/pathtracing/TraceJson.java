package pathtracing;

import com.google.gson.JsonObject;
import java.util.Locale;
import net.minecraft.world.phys.Vec3;

/** Decimal values for inspection, raw IEEE bits for the movement judge. */
final class TraceJson {
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
}
