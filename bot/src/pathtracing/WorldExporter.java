package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.List;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.message.v1.ClientReceiveMessageEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.attribute.EnvironmentAttributes;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.level.block.BedBlock;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.BubbleColumnBlock;
import net.minecraft.world.level.block.HayBlock;
import net.minecraft.world.level.block.HoneyBlock;
import net.minecraft.world.level.block.LadderBlock;
import net.minecraft.world.level.block.LiquidBlock;
import net.minecraft.world.level.block.PowderSnowBlock;
import net.minecraft.world.level.block.EndPortalBlock;
import net.minecraft.world.level.block.EndGatewayBlock;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.block.ScaffoldingBlock;
import net.minecraft.world.level.block.SlimeBlock;
import net.minecraft.world.level.block.TrapDoorBlock;
import net.minecraft.world.level.block.WebBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.gamerules.GameRules;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.CollisionContext;

/** World data comes from the loaded client level, with the dumper's entity contexts. */
final class WorldExporter {
    private record Request(String name, Path path, BlockPos min, BlockPos max, long sent) {}
    private static Request pending;

    static void register() {
        // Remote ClientLevel has no game rules. Read the server's query reply through the client.
        ClientReceiveMessageEvents.GAME.register((message, overlay) -> {
            if (pending == null || !(message.getContents() instanceof TranslatableContents text)
                || !text.getKey().equals("commands.gamerule.query")) return;
            Object[] args = text.getArgs();
            if (args.length != 2 || !args[0].equals(GameRules.NATURAL_HEALTH_REGENERATION.id())) return;
            String value = args[1].toString();
            if (!value.equals("true") && !value.equals("false")) return;
            Request request = pending;
            pending = null;
            Minecraft mc = Minecraft.getInstance();
            try {
                Bot.reply(mc, write(mc, request, Boolean.parseBoolean(value)));
            } catch (IOException | RuntimeException e) {
                Bot.reply(mc, "Recorder/dumper: " + e.getMessage());
            }
        });
        ClientTickEvents.END_CLIENT_TICK.register(mc -> {
            if (pending != null && System.nanoTime() - pending.sent() > 15_000_000_000L) {
                pending = null;
                Bot.reply(mc, "Recorder/dumper: Export game-rule query timed out; operator permission is required.");
            }
        });
        ClientPlayConnectionEvents.DISCONNECT.register((handler, mc) -> pending = null);
    }

    static void export(Minecraft mc, String name, BlockPos first, BlockPos last) throws IOException {
        if (Recorder.active() || pending != null) throw new IllegalStateException("Recording, replay or export is running.");
        Path path = Recorder.file(mc, "fixtures", name).resolveSibling(name + ".world.jsonl");
        if (Files.exists(path) || Files.exists(temporary(path))) throw new IllegalStateException("Export output already exists: " + path);
        BlockPos min = new BlockPos(Math.min(first.getX(), last.getX()), Math.min(first.getY(), last.getY()), Math.min(first.getZ(), last.getZ()));
        BlockPos max = new BlockPos(Math.max(first.getX(), last.getX()), Math.max(first.getY(), last.getY()), Math.max(first.getZ(), last.getZ()));
        pending = new Request(name, path, min, max, System.nanoTime());
        mc.getConnection().sendCommand("gamerule " + GameRules.NATURAL_HEALTH_REGENERATION.id());
    }

    private static Path temporary(Path path) { return path.resolveSibling(path.getFileName() + ".tmp"); }

    private static String write(Minecraft mc, Request request, boolean naturalRegeneration) throws IOException {
        if (Recorder.active()) throw new IllegalStateException("A recording or replay started during export.");
        var level = mc.level;
        if (level == null || mc.player == null) throw new IllegalStateException("Export lost player or level.");
        for (int x = request.min().getX() >> 4; x <= request.max().getX() >> 4; x++) {
            for (int z = request.min().getZ() >> 4; z <= request.max().getZ() >> 4; z++) {
                if (!level.hasChunk(x, z)) throw new IllegalStateException("Export contains an unloaded client chunk: " + x + "," + z);
            }
        }
        var player = mc.player;
        Vec3 position = player.position();
        AABB bounds = player.getBoundingBox();
        Pose pose = player.getPose();
        Input input = player.input.keyPresses;
        double fallDistance = player.fallDistance;
        ItemStack mainHand = player.getMainHandItem();
        Files.createDirectories(request.path().getParent());
        int count = 0;
        try (BufferedWriter writer = Files.newBufferedWriter(temporary(request.path()), StandardCharsets.UTF_8, StandardOpenOption.CREATE_NEW)) {
            JsonObject world = new JsonObject();
            world.addProperty("kind", "world");
            world.addProperty("minY", level.getMinY());
            world.addProperty("difficulty", level.getDifficulty().getId());
            world.addProperty("naturalRegeneration", naturalRegeneration);
            world.addProperty("fastLava", level.environmentAttributes().getDimensionValue(EnvironmentAttributes.FAST_LAVA));
            writer.write(world.toString()); writer.write('\n');
            for (BlockPos pos : BlockPos.betweenClosed(request.min(), request.max())) {
                BlockState state = level.getBlockState(pos);
                if (state.isAir()) continue;
                JsonObject out = new JsonObject();
                out.addProperty("kind", "block");
                JsonArray at = new JsonArray();
                at.add(pos.getX()); at.add(pos.getY()); at.add(pos.getZ());
                out.add("at", at);
                out.addProperty("block", BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
                out.addProperty("state", state.toString());
                out.add("boxes", boxes(state.getCollisionShape(level, pos, CollisionContext.empty()).toAabbs()));
                String[] names = {"aboveBoxes", "belowBoxes", "sneakBoxes", "holdingScaffoldingBoxes"};
                for (int i = 0; i < names.length; i++) {
                    player.setItemInHand(InteractionHand.MAIN_HAND, i == 3 ? new ItemStack(Items.SCAFFOLDING) : mainHand);
                    player.setPose(i == 2 ? Pose.CROUCHING : Pose.STANDING);
                    player.input.keyPresses = new Input(false, false, false, false, false, i == 2, false);
                    player.fallDistance = 0;
                    player.setPos(pos.getX() + 0.5, pos.getY() + (i == 1 ? -2 : 1), pos.getZ() + 0.5);
                    out.add(names[i], boxes(state.getCollisionShape(level, pos, CollisionContext.of(player)).toAabbs()));
                }
                player.setItemInHand(InteractionHand.MAIN_HAND, mainHand);
                player.setPose(pose);
                player.input.keyPresses = input;
                player.fallDistance = fallDistance;
                player.setPos(position);
                player.setBoundingBox(bounds);
                out.add("insideBoxes", boxes(state.getEntityInsideCollisionShape(level, pos, player).toAabbs()));
                properties(out, state);
                out.addProperty("suffocating", state.isSuffocating(level, pos));
                var fluid = state.getFluidState();
                out.addProperty("fluid", fluid.is(FluidTags.WATER) ? "water" : fluid.is(FluidTags.LAVA) ? "lava" : "empty");
                TraceJson.number(out, "fluidHeight", fluid.getHeight(level, pos));
                out.add("flow", TraceJson.vector(fluid.getFlow(level, pos)));
                writer.write(out.toString()); writer.write('\n');
                count++;
            }
        } finally {
            player.setItemInHand(InteractionHand.MAIN_HAND, mainHand);
            player.input.keyPresses = input;
            player.setPose(pose);
            player.fallDistance = fallDistance;
            player.setPos(position);
            player.setBoundingBox(bounds);
        }
        Files.move(temporary(request.path()), request.path());
        return "Exported " + request.name() + ": " + count + " blocks.";
    }

    private static void properties(JsonObject out, BlockState state) {
        Block block = state.getBlock();
        TraceJson.number(out, "friction", block.getFriction());
        TraceJson.number(out, "speedFactor", block.getSpeedFactor());
        TraceJson.number(out, "jumpFactor", block.getJumpFactor());
        TraceJson.number(out, "bounce", block.getBounceRestitution());
        out.addProperty("climbable", state.is(BlockTags.CLIMBABLE));
        out.addProperty("fence", state.is(BlockTags.FENCES));
        out.addProperty("wallOrGate", state.is(BlockTags.WALLS) || state.is(BlockTags.FENCE_GATES));
        out.addProperty("suppressesBounce", state.is(BlockTags.SUPPRESSES_BOUNCE));
        out.addProperty("resetsFallDistance", state.is(BlockTags.FALL_DAMAGE_RESETTING));
        out.addProperty("air", state.isAir());
        out.addProperty("scaffolding", block instanceof ScaffoldingBlock);
        out.addProperty("slime", block instanceof SlimeBlock);
        out.addProperty("bed", block instanceof BedBlock);
        out.addProperty("endPortalOrGateway", block instanceof EndPortalBlock || block instanceof EndGatewayBlock);
        out.addProperty("waterBlock", block instanceof LiquidBlock && state.getFluidState().is(FluidTags.WATER) || block instanceof BubbleColumnBlock);
        String inside = block instanceof WebBlock ? "cobweb" : block instanceof PowderSnowBlock ? "powderSnow" : block instanceof HoneyBlock ? "honey" : "none";
        boolean modeledInside = block instanceof WebBlock || block instanceof PowderSnowBlock || block instanceof HoneyBlock;
        boolean modeledFall = block instanceof HayBlock || block instanceof HoneyBlock || block instanceof BedBlock
            || block instanceof SlimeBlock || block instanceof PowderSnowBlock;
        if ((!modeledInside && overrides(block, "entityInside", BlockBehaviour.class))
            || (!modeledFall && overrides(block, "fallOn", Block.class))) inside = "unmodeled";
        out.addProperty("inside", inside);
        // These fallOn hooks have no data getter. Values follow their 26.2 bytecode.
        TraceJson.number(out, "fallOnMultiplier", block instanceof HayBlock || block instanceof HoneyBlock ? 0.2F : 1.0F);
        if (block instanceof LadderBlock) out.addProperty("ladderFacing", state.getValue(LadderBlock.FACING).get3DDataValue());
        if (block instanceof TrapDoorBlock && state.getValue(TrapDoorBlock.OPEN)) {
            out.addProperty("openTrapdoorFacing", state.getValue(TrapDoorBlock.FACING).get3DDataValue());
        }
    }

    private static boolean overrides(Block block, String hook, Class<?> base) {
        // Walk inherited overrides too. Unknown hooks must never assert "none".
        for (Class<?> type = block.getClass(); type != base; type = type.getSuperclass()) {
            for (var method : type.getDeclaredMethods()) {
                if (method.getName().equals(hook)) return true;
            }
        }
        return false;
    }

    private static JsonArray boxes(List<AABB> values) {
        JsonArray out = new JsonArray();
        for (AABB box : values) {
            JsonArray row = new JsonArray();
            row.add(box.minX); row.add(box.minY); row.add(box.minZ);
            row.add(box.maxX); row.add(box.maxY); row.add(box.maxZ);
            out.add(row);
        }
        return out;
    }
}
