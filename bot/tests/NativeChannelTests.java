package pathtracing;

import com.google.gson.*;
import java.nio.file.*;
import java.util.*;

public final class NativeChannelTests {
    private static void number(JsonObject o,String key,double value,boolean wide) {
        o.addProperty(key,value);
        o.addProperty(key+"Bits",wide?Long.toHexString(Double.doubleToRawLongBits(value)):Integer.toHexString(Float.floatToRawIntBits((float)value)));
    }
    private static JsonObject vector(double x,double y,double z) {
        JsonObject v=new JsonObject(); number(v,"x",x,true); number(v,"y",y,true); number(v,"z",z,true); return v;
    }
    private static StateBits.Observation exhaustive() {
        JsonObject p=new JsonObject(), input=new JsonObject();
        p.addProperty("tickCount",123); p.add("inputs",input); p.addProperty("pose","SWIMMING");
        for(String key:new String[]{"forward","back","left","right","jump","sneak","sprint"}) input.addProperty(key,true);
        number(input,"yaw",3.5,false); number(input,"pitch",-2,false); number(p,"yRot",4.5,false); number(p,"xRot",-3,false);
        p.addProperty("hurtRemainingTicks",-7); number(p,"lastRawHit",2.25,false);
        p.addProperty("foodLevel",7); p.addProperty("foodTickTimer",1234); p.addProperty("remainingFireTicks",-20); p.addProperty("noJumpDelay",3); p.addProperty("ticksFrozen",2);
        for(String key:new String[]{"onGround","horizontalCollision","verticalCollision","verticalCollisionBelow","minorHorizontalCollision","isSprinting","sprintSpeedModifier","noPhysics","isShiftKeyDown","isInWater","isInLava","onClimbable","onGroundNoBlocks","crouching","swimming","underwater","eyeInWater","wasEyeInWater","inPowderSnow","wasInPowderSnow","leatherBoots","fallFlying","riding","boat","effects","flying","isUsingItem"}) p.addProperty(key,true);
        for(String key:new String[]{"health","saturationLevel","exhaustion","speed","stepHeight","frictionModifier","airDragModifier","waterMovementEfficiency","movementEfficiency","sneakingSpeed","useItemSpeedMultiplier"}) number(p,key,1.25,false);
        p.addProperty("useItemCanSprint",false);
        for(String key:new String[]{"fallDistance","waterHeight","lavaHeight","movementSpeedBase","jumpStrength","gravity","safeFallDistance","fallDamageMultiplier","bounciness"}) number(p,key,1.25,true);
        p.add("position",vector(-0.0,-60,123.25));
        for(String key:new String[]{"velocity","stuckSpeedMultiplier","movementThisTick"}) p.add(key,vector(1,2,3));
        JsonArray support=new JsonArray(); for(int i:new int[]{-1,-61,2}) support.add(i); p.add("mainSupportingBlock",support);
        JsonArray modifiers=new JsonArray(), row=new JsonArray(); row.add("minecraft:é"); row.add(1.25); row.add(2); modifiers.add(row); p.add("speedModifiers",modifiers);
        return StateBits.encode(p);
    }
    private static void check(boolean value,String why) { if(!value) throw new AssertionError(why); }
    private static void echo(StateBits.Observation observation) {
        long[] result=NativeChannel.roundTrip(observation.bits(),observation.known(),observation.provenance());
        int n=observation.bits().length;
        check(result[0]==0 && result[1]==n && result.length==2+n*3,"roundtrip envelope");
        check(Arrays.equals(Arrays.copyOfRange(result,2,2+n),observation.bits()),"raw state bits");
        check(Arrays.equals(Arrays.copyOfRange(result,2+n,2+n*2),observation.known()),"known mask");
        check(Arrays.equals(Arrays.copyOfRange(result,2+n*2,result.length),observation.provenance()),"provenance mask");
    }
    private static void missing(StateBits.Observation o) {
        long[] r=NativeChannel.RequestGoal(o.bits(),o.known(),o.provenance(),new long[]{1,1,0,0,0,1},new long[]{0,-61,0},1,1);
        check(r[0]==3,"missing data must refuse synchronously");
    }
    /** KIT_PLAN action records decode word for word; only an issued action's own cell (or its door half) is tolerated. */
    private static void kitActions() {
        KitAction place=new KitAction(5,1,KitAction.PLACE,8,1,12,1,10.5f,60f,1), door=new KitAction(2,3,KitAction.INTERACT,3,1,4,2,-90f,12f,1);
        long[] wire=new long[1+2*KitAction.WORDS]; int o=1;
        for(KitAction a:List.of(place,door)) for(long v:new long[]{a.tick(),a.slot(),a.opcode(),a.x(),a.y(),a.z(),a.face(),Float.floatToRawIntBits(a.yaw()),Float.floatToRawIntBits(a.pitch()),a.hold()}) wire[o++]=v;
        check(KitAction.read(wire,1,2).equals(List.of(place,door)),"action records decode");
        List<KitAction> plan=List.of(place,door);
        check(!KitAction.ours(plan,4,8,1,12) && KitAction.ours(plan,5,8,1,12),"an edit is ours once its click is issued");
        check(!KitAction.ours(plan,9,8,2,12) && !KitAction.ours(plan,9,9,1,12),"a neighbouring edit stops the flight");
        check(KitAction.ours(plan,2,3,2,4) && KitAction.ours(plan,2,3,0,4) && !KitAction.ours(plan,2,3,3,4),"a door moves both halves");
    }
    public static void main(String[] args) throws Exception {
        check(NativeChannel.loadFailure()==null,"native load: "+NativeChannel.loadFailure());
        StateBits.Observation all=exhaustive();
        long[] expected=Arrays.stream(Files.readString(Path.of(args[0],"native-state.hex")).trim().split("\\s+")).mapToLong(w->Long.parseUnsignedLong(w,16)).toArray();
        check(Arrays.equals(expected,all.bits()),"Java encoder differs from Rust exhaustive field witness");
        check(Arrays.stream(all.known()).allMatch(v->v==1),"exhaustive fixture unexpectedly unknown");
        echo(all);
        long[] event={1,42,123,7,1};
        check(NativeChannel.ObserveEvent(event,all.bits(),all.known(),all.provenance())[0]==0,"event accepted");
        long[] saved=NativeChannel.ObserveEvent(new long[]{2},new long[0],new long[0],new long[0]);
        check(saved[0]==0 && saved[1]==42 && saved[2]==event.length,"native event retained");
        int start=4+event.length;
        check(Arrays.equals(Arrays.copyOfRange(saved,start,start+all.bits().length),all.bits()),"native retained observed bits");
        for(int i=0;i<all.bits().length;i++) {
            long[] k=all.known().clone(), p=all.provenance().clone(); k[i]=0; p[i]=0;
            StateBits.Observation partial=new StateBits.Observation(all.bits(),k,p); echo(partial); missing(partial);
        }
        long[] malformed=NativeChannel.roundTrip(all.bits(),Arrays.copyOf(all.known(),all.known().length-1),all.provenance());
        check(malformed[0]==3,"short mask refused");
        JsonObject header=JsonParser.parseString(Files.readString(Path.of(args[0],"recorded-player.json"))).getAsJsonObject();
        StateBits.Observation recorded=StateBits.encode(header.getAsJsonObject("initialPlayer"));
        echo(recorded); missing(recorded);
        StringBuilder encoded=new StringBuilder();
        for(long[] values:new long[][]{recorded.bits(),recorded.known(),recorded.provenance()}) {
            for(long v:values) encoded.append(Long.toUnsignedString(v,16)).append(' '); encoded.append('\n');
        }
        if(args.length>1) Files.writeString(Path.of(args[1]),encoded);
        kitActions();
        NativeClockTests.main(new String[0]);
        System.out.println("PASS: every field/mask Java -> Rust -> Java; every unknown refused; pinned recording preserved");
    }
}
