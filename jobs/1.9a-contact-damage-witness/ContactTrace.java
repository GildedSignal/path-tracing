package pathtracing.contact;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.StringJoiner;
import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.gamerules.GameRules;

public final class ContactTrace implements DedicatedServerModInitializer {
    private static final int TICKS = 32;
    private static final BlockPos PAD = new BlockPos(4112, -61, 4100);
    private static final HexFormat HEX = HexFormat.of();
    private static final StringBuilder rows = new StringBuilder();
    private static Path output;
    private static String target;
    private static ServerPlayer player;
    private static ServerPlayer priorPlayer;
    private static String prior;
    private static int seq;
    private static int completed;
    private static long startedNanos;
    private static boolean inTick;
    private static boolean inStep;
    private static boolean sealed;
    private static Damage pending;

    private record Damage(DamageSource source, float amount, boolean contact, float health, int cooldown) {}

    @Override
    public void onInitializeServer() {
        String trial = System.getProperty("pathtracing.contact.case");
        if (trial == null) return;
        target = switch (trial) {
            case "magma-standing", "magma-sneaking" -> "minecraft:magma_block";
            case "netherrack-standing" -> "minecraft:netherrack";
            default -> throw new IllegalArgumentException("Unknown contact witness case " + trial);
        };
        Path directory = FabricLoader.getInstance().getGameDir();
        if (!Files.isRegularFile(directory.resolve("contact-witness.disposable"))) {
            throw new IllegalStateException("Contact witness requires a marked disposable server");
        }
        output = directory.resolve(trial + ".tsv");
        if (Files.exists(output)) throw new IllegalStateException("Contact evidence already exists at " + output);
        rows.append(fields("contact-v1", trial, "PathBot")).append('\n');
    }

    public static void begin(ServerPlayer candidate) {
        if (output == null || sealed || !candidate.getScoreboardName().equals("PathBot")) return;
        if (player == null) {
            if (!atPad(candidate) || !block(candidate, PAD).equals(target)) return;
            player = candidate;
            startedNanos = System.nanoTime();
            if (priorPlayer == player && prior != null) append(prior);
        }
        if (candidate != player) return;
        if (inTick || inStep || pending != null) row("X", "unclosed-event-at-begin");
        inTick = true;
        append(frame("B", player));
    }

    public static void end(ServerPlayer candidate) {
        if (output == null || sealed) return;
        if (player == null) {
            if (candidate.getScoreboardName().equals("PathBot")) {
                priorPlayer = candidate;
                prior = atPad(candidate) ? frame("P", candidate) : null;
            }
            return;
        }
        if (candidate != player) return;
        if (!inTick || inStep || pending != null) row("X", "unclosed-event-at-end");
        append(frame("E", player));
        inTick = false;
        if (++completed == TICKS) {
            row("Z", TICKS, System.nanoTime() - startedNanos);
            sealed = true;
            try {
                Files.writeString(output, rows, StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE);
            } catch (IOException error) {
                System.err.println("Cannot save contact witness " + output + " (" + error + ")");
            }
        }
    }

    public static void stepBegin(ServerPlayer candidate) {
        if (!observing(candidate)) return;
        if (!inTick || inStep) row("X", "ambiguous-step-begin");
        inStep = true;
        append(frame("C", candidate));
    }

    public static void stepEnd(ServerPlayer candidate) {
        if (!observing(candidate)) return;
        if (!inTick || !inStep || pending != null) row("X", "ambiguous-step-end");
        inStep = false;
    }

    public static void damageBegin(ServerPlayer candidate, DamageSource source, float amount) {
        if (player == null && candidate == priorPlayer) prior = null;
        if (!observing(candidate)) return;
        if (pending != null) row("X", "nested-damage");
        pending = new Damage(source, amount, inTick && inStep, candidate.getHealth(), candidate.invulnerableTime);
    }

    public static void damageEnd(ServerPlayer candidate, DamageSource source, boolean accepted) {
        if (!observing(candidate)) return;
        if (pending == null || pending.source != source) {
            row("X", "unpaired-damage-return");
            return;
        }
        String key = source.typeHolder().unwrapKey().map(k -> k.identifier().toString()).orElse("unkeyed");
        row("R", candidate.level().getGameTime(), candidate.tickCount, pending.contact, key,
            bits(pending.amount), accepted, bits(pending.health), bits(candidate.getHealth()),
            pending.cooldown, candidate.invulnerableTime);
        pending = null;
    }

    private static boolean observing(ServerPlayer candidate) {
        return !sealed && candidate == player;
    }

    private static boolean atPad(ServerPlayer p) {
        return p.level().dimension().equals(Level.OVERWORLD)
            && p.getX() == 4112.5 && p.getY() == -60.0 && p.getZ() == 4100.5;
    }

    private static String block(ServerPlayer p, BlockPos pos) {
        return BuiltInRegistries.BLOCK.getKey(p.level().getBlockState(pos).getBlock()).toString();
    }

    private static String frame(String tag, ServerPlayer p) {
        var level = p.level();
        var clock = level.tickRateManager();
        var input = p.getLastClientInput();
        boolean move = input.forward() || input.backward() || input.left() || input.right()
            || input.jump() || input.sprint();
        boolean bare = Arrays.stream(EquipmentSlot.values()).allMatch(slot -> p.getItemBySlot(slot).isEmpty());
        var support = p.mainSupportingBlockPos;
        // The observed invocation uses the legacy selector.
        BlockPos contact = p.getOnPosLegacy();
        return fields(tag, level.getGameTime(), p.tickCount, p.getX(), p.getY(), p.getZ(),
            p.onGround(), p.getPose(), p.isSteppingCarefully(), input.shift(), move,
            support.filter(PAD::equals).isPresent(), contact.equals(PAD), block(p, contact),
            p.gameMode(), level.getDifficulty(), level.getGameRules().get(GameRules.NATURAL_HEALTH_REGENERATION),
            level.getGameRules().get(GameRules.FIRE_DAMAGE),
            p.getActiveEffects().size(), bare, bits(p.getMaxHealth()), bits(p.getAbsorptionAmount()),
            p.isInvulnerable() || p.getAbilities().invulnerable || p.isChangingDimension(),
            p.connection.hasClientLoaded(),
            level.dimension().equals(Level.OVERWORLD), clock.tickrate(), clock.isFrozen(),
            bits(p.getHealth()), p.invulnerableTime);
    }

    private static String bits(float value) {
        return HEX.toHexDigits(Float.floatToRawIntBits(value));
    }

    private static String fields(Object... values) {
        StringJoiner line = new StringJoiner("\t");
        for (Object value : values) line.add(value.toString());
        return line.toString();
    }

    private static void row(Object... values) {
        append(fields(values));
    }

    private static void append(String line) {
        rows.append(++seq).append('\t').append(line).append('\n');
    }
}
