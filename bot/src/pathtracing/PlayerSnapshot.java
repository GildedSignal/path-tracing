package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.client.Minecraft;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.item.Items;
import pathtracing.mixin.EntityStateAccess;
import pathtracing.mixin.FoodStateAccess;
import pathtracing.mixin.LivingStateAccess;

/** Actual client snapshot; each caller identifies its input boundary. */
final class PlayerSnapshot {
    /** Effects a plan may ignore. The first line acts on nothing the law observes for a player (drawing,
     *  mobs, raids, loot). The second only adds margin, food, health or less damage, so no certified move
     *  is lost under it (precision only where a decision can flip); the server keeps the real counts. */
    private static final java.util.Set<String> INERT_EFFECTS = java.util.Set.of("night_vision", "glowing", "invisibility",
        "luck", "unluck", "bad_omen", "trial_omen", "raid_omen", "hero_of_the_village", "nausea", "darkness",
        "saturation", "regeneration", "instant_health", "absorption", "health_boost", "resistance", "fire_resistance", "water_breathing");
    static long[] context(Minecraft mc,long at) {
        JsonObject facts = new JsonObject();
        facts.addProperty("dimension",mc.level.dimension().identifier().toString());
        facts.addProperty("player",mc.player.getUUID().toString());
        facts.addProperty("mode",mc.gameMode.getPlayerMode().getName());
        facts.addProperty("pose",mc.player.getPose().name());
        var slots = new java.util.ArrayList<net.minecraft.world.item.ItemStack>();
        for (int slot=TrackInventory.FIRST;slot<TrackInventory.FIRST+TrackInventory.SIZE;slot++) slots.add(mc.player.inventoryMenu.getSlot(slot).getItem().copy());
        facts.add("kit",TrackInventoryCodec.encode(slots,mc.player.getInventory().getSelectedSlot(),mc.level.registryAccess()).json());
        byte[] bytes=facts.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8);
        long[] meta=new long[7+bytes.length];
        meta[0]=ResidentSource.session; meta[1]=at; meta[2]=0;
        meta[3]=0; meta[4]=0; // No invented authoritative server tick.
        meta[5]=ResidentSource.revision; meta[6]=bytes.length;
        for(int i=0;i<bytes.length;i++) meta[7+i]=Byte.toUnsignedLong(bytes[i]);
        return meta;
    }
    /** R0 cold observation, without replay/settling or an invented server row. */
    static StateBits.Observation observed(Minecraft mc) {
        JsonObject out = capture(mc);
        var p = mc.player;
        out.addProperty("noPhysics",p.noPhysics);
        out.addProperty("remainingFireTicks",p.getRemainingFireTicks());
        out.addProperty("fallFlying",p.isFallFlying());
        out.addProperty("riding",p.isPassenger());
        out.addProperty("boat",p.getVehicle() instanceof net.minecraft.world.entity.vehicle.boat.AbstractBoat);
        // Effects the law observes: these change nothing about the body, its resources or its
        // breaking, only what is drawn or how mobs and villagers treat it, so they are no effect.
        out.addProperty("effects",p.getActiveEffects().stream().anyMatch(e -> !INERT_EFFECTS.contains(
            e.getEffect().unwrapKey().map(k -> k.identifier().getPath()).orElse(""))));
        out.addProperty("flying",p.getAbilities().flying);
        out.addProperty("mainSupportingBlockKnown",true);
        // try_tick overwrites movement_this_tick before its first read.
        out.addProperty("movementThisTickProjected",true);
        if (!p.isUsingItem()) {
            out.addProperty("useItemSpeedMultiplierAbsent",true);
            out.addProperty("useItemCanSprintAbsent",true);
        }
        // Local FoodData and the hurt animation are not the server's exhaustion, food timer and hurt cooldown:
        // they go as projections (precision only where a decision can flip; the server corrects the rest).
        for (String key : new String[]{"exhaustion","foodTickTimer","hurtRemainingTicks","lastRawHit"}) out.addProperty(key + "Projected",true);
        return StateBits.encode(out);
    }

    static JsonObject capture(Minecraft mc) {
        var p = mc.player;
        var entity = (EntityStateAccess) p;
        var food = (FoodStateAccess) p.getFoodData();
        JsonObject out = new JsonObject();
        out.addProperty("kind", "player");
        out.addProperty("tickCount", p.tickCount);
        out.add("position", TraceJson.vector(p.position()));
        out.add("velocity", TraceJson.vector(p.getDeltaMovement()));
        out.add("inputs", TickInput.capture(mc).json());
        TraceJson.number(out.getAsJsonObject("inputs"),"yaw",p.getYRot());
        TraceJson.number(out.getAsJsonObject("inputs"),"pitch",p.getXRot());
        TraceJson.number(out, "yRot", p.getYRot());
        TraceJson.number(out, "xRot", p.getXRot());
        out.addProperty("onGround", p.onGround());
        out.addProperty("horizontalCollision", p.horizontalCollision);
        out.addProperty("verticalCollision", p.verticalCollision);
        out.addProperty("verticalCollisionBelow", p.verticalCollisionBelow);
        out.addProperty("minorHorizontalCollision", p.minorHorizontalCollision);
        out.addProperty("isSprinting", p.isSprinting());
        out.addProperty("isUsingItem", p.isUsingItem());
        out.addProperty("isShiftKeyDown", p.isShiftKeyDown());
        out.addProperty("pose", p.getPose().name());
        out.addProperty("crouching", p.isCrouching());
        out.addProperty("swimming", p.isSwimming());
        out.addProperty("underwater", p.isUnderWater());
        out.addProperty("eyeInWater", p.isEyeInFluid(FluidTags.WATER));
        out.addProperty("wasEyeInWater", entity.pathtracing$wasEyeInWater());
        out.addProperty("isInWater", p.isInWater());
        out.addProperty("isInLava", p.isInLava());
        out.addProperty("onClimbable", p.onClimbable());
        out.addProperty("leatherBoots", p.getItemBySlot(EquipmentSlot.FEET).is(Items.LEATHER_BOOTS));
        out.addProperty("inPowderSnow", p.isInPowderSnow);
        out.addProperty("wasInPowderSnow", p.wasInPowderSnow);
        out.addProperty("ticksFrozen", p.getTicksFrozen());
        out.addProperty("onGroundNoBlocks", entity.pathtracing$onGroundNoBlocks());
        out.addProperty("noJumpDelay", ((LivingStateAccess) p).pathtracing$noJumpDelay());
        out.addProperty("foodLevel", p.getFoodData().getFoodLevel());
        out.addProperty("foodTickTimer", food.pathtracing$foodTickTimer());
        out.addProperty("hurtRemainingTicks", MinecraftApi.damageCooldown(p));
        TraceJson.number(out, "lastRawHit", ((LivingStateAccess) p).pathtracing$lastRawHit());
        TraceJson.number(out, "health", p.getHealth());
        TraceJson.number(out, "saturationLevel", p.getFoodData().getSaturationLevel());
        TraceJson.number(out, "exhaustion", food.pathtracing$exhaustion());
        TraceJson.number(out, "speed", p.getSpeed());
        TraceJson.number(out, "stepHeight", (float) p.getAttributeValue(Attributes.STEP_HEIGHT));
        TraceJson.number(out, "frictionModifier", (float) p.getAttributeValue(Attributes.FRICTION_MODIFIER));
        TraceJson.number(out, "airDragModifier", (float) p.getAttributeValue(Attributes.AIR_DRAG_MODIFIER));
        TraceJson.number(out, "waterMovementEfficiency", (float) p.getAttributeValue(Attributes.WATER_MOVEMENT_EFFICIENCY));
        TraceJson.number(out, "movementEfficiency", (float) p.getAttributeValue(Attributes.MOVEMENT_EFFICIENCY));
        TraceJson.number(out, "sneakingSpeed", (float) p.getAttributeValue(Attributes.SNEAKING_SPEED));
        TraceJson.number(out, "movementSpeedBase", p.getAttributeBaseValue(Attributes.MOVEMENT_SPEED));
        out.add("speedModifiers", TraceJson.speedModifiers(p.getAttribute(Attributes.MOVEMENT_SPEED)));
        out.addProperty("sprintSpeedModifier", TraceJson.sprintSpeedModifier(p.getAttribute(Attributes.MOVEMENT_SPEED)));
        TraceJson.number(out, "jumpStrength", p.getAttributeValue(Attributes.JUMP_STRENGTH));
        TraceJson.number(out, "gravity", p.getAttributeValue(Attributes.GRAVITY));
        TraceJson.number(out, "safeFallDistance", p.getAttributeValue(Attributes.SAFE_FALL_DISTANCE));
        TraceJson.number(out, "fallDamageMultiplier", p.getAttributeValue(Attributes.FALL_DAMAGE_MULTIPLIER));
        TraceJson.number(out, "bounciness", p.getAttributeValue(Attributes.BOUNCINESS));
        TraceJson.number(out, "fallDistance", p.fallDistance);
        TraceJson.number(out, "waterHeight", p.getFluidHeight(FluidTags.WATER));
        TraceJson.number(out, "lavaHeight", p.getFluidHeight(FluidTags.LAVA));
        out.add("stuckSpeedMultiplier", TraceJson.vector(entity.pathtracing$stuckSpeedMultiplier()));
        p.mainSupportingBlockPos.ifPresent(pos -> {
            JsonArray at = new JsonArray();
            at.add(pos.getX()); at.add(pos.getY()); at.add(pos.getZ());
            out.add("mainSupportingBlock", at);
        });
        return out;
    }
}
