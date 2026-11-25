import java.nio.file.*;
import java.util.*;
import java.io.*;
import java.util.zip.*;
import java.util.regex.*;
import java.util.stream.*;
import java.awt.image.BufferedImage;
import javax.imageio.ImageIO;
import java.security.MessageDigest;

/** Offline architectural preview. Coordinates and commands use Minecraft's absolute block grid. */
public class Preview {
    record Block(String id, Map<String,String> states) {
        static Block parse(String text) {
            int p = text.indexOf('[');
            String id = (p < 0 ? text : text.substring(0,p)).replaceFirst("^minecraft:", "");
            Map<String,String> states = new HashMap<>();
            if (p >= 0) {
                if (!text.endsWith("]")) throw new IllegalArgumentException("Unclosed state: " + text);
                for (String part : text.substring(p+1,text.length()-1).split(",")) {
                    String[] pair = part.split("=", 2);
                    if (pair.length != 2) throw new IllegalArgumentException("Bad state: " + part);
                    states.put(pair[0], pair[1]);
                }
            }
            return new Block(id, Map.copyOf(states));
        }
    }
    static final Block AIR = Block.parse("air"), GRASS = Block.parse("grass_block"), DIRT = Block.parse("dirt");
    static class World {
        final int x0,y0,z0,x1,y1,z1,nx,ny,nz;
        final int[] cells;
        final List<Block> palette = new ArrayList<>();
        final Map<Block,Integer> ids = new HashMap<>();
        World(int x0,int y0,int z0,int x1,int y1,int z1) {
            this.x0=Math.min(x0,x1); this.y0=Math.min(y0,y1); this.z0=Math.min(z0,z1);
            this.x1=Math.max(x0,x1); this.y1=Math.max(y0,y1); this.z1=Math.max(z0,z1);
            nx=Math.addExact(Math.subtractExact(this.x1,this.x0),1);
            ny=Math.addExact(Math.subtractExact(this.y1,this.y0),1);
            nz=Math.addExact(Math.subtractExact(this.z1,this.z0),1);
            cells=new int[Math.multiplyExact(Math.multiplyExact(nx,ny),nz)];
            id(AIR); id(GRASS); id(DIRT);
            for(int y=this.y0;y<=this.y1;y++) {
                int v=y < -61 ? 2 : y == -61 ? 1 : 0;
                Arrays.fill(cells,(y-this.y0)*nx*nz,(y-this.y0+1)*nx*nz,v);
            }
        }
        int id(Block b) { return ids.computeIfAbsent(b,k->{palette.add(k); return palette.size()-1;}); }
        boolean inside(int x,int y,int z) { return x>=x0&&x<=x1&&y>=y0&&y<=y1&&z>=z0&&z<=z1; }
        int index(int x,int y,int z) { return ((y-y0)*nz+z-z0)*nx+x-x0; }
        Block get(int x,int y,int z) {
            return inside(x,y,z) ? palette.get(cells[index(x,y,z)]) : y < -61 ? DIRT : y == -61 ? GRASS : AIR;
        }
        void fill(int xa,int ya,int za,int xb,int yb,int zb,Block b) {
            int a=Math.max(x0,Math.min(xa,xb)), c=Math.min(x1,Math.max(xa,xb));
            int d=Math.max(y0,Math.min(ya,yb)), e=Math.min(y1,Math.max(ya,yb));
            int f=Math.max(z0,Math.min(za,zb)), g=Math.min(z1,Math.max(za,zb));
            if(a>c||d>e||f>g)return;
            int v=id(b);
            for(int y=d;y<=e;y++) for(int z=f;z<=g;z++) Arrays.fill(cells,index(a,y,z),index(c,y,z)+1,v);
        }
        void replay(Path path) throws IOException {
            int line=0;
            for(String raw:Files.readAllLines(path)) {
                line++;
                String s=raw.replace("\ufeff", "").strip();
                if(s.isEmpty()||s.startsWith("#"))continue;
                if(s.startsWith("/")) s=s.substring(1);
                String[] p=s.split("\\s+");
                try {
                    if(p[0].equals("setblock") && p.length==5) {
                        int x=Integer.parseInt(p[1]),y=Integer.parseInt(p[2]),z=Integer.parseInt(p[3]);
                        fill(x,y,z,x,y,z,Block.parse(p[4]));
                    } else if(p[0].equals("fill") && p.length==8) {
                        fill(Integer.parseInt(p[1]),Integer.parseInt(p[2]),Integer.parseInt(p[3]),
                             Integer.parseInt(p[4]),Integer.parseInt(p[5]),Integer.parseInt(p[6]),Block.parse(p[7]));
                    } else throw new IllegalArgumentException("Unsupported command: " + s);
                } catch(RuntimeException ex) { throw new IllegalArgumentException(path+":"+line+": "+ex.getMessage(),ex); }
            }
        }
    }
    // Assets stay inside the jar. This small JSON reader handles model inheritance and blockstates.
    static class Json {
        final String s; int p;
        Json(String s){this.s=s;}
        void ws(){while(p<s.length()&&Character.isWhitespace(s.charAt(p)))p++;}
        Object value(){ws();char c=s.charAt(p++);
            if(c=='{'){Map<String,Object>m=new LinkedHashMap<>();ws();if(s.charAt(p)=='}'){p++;return m;}do{ws();String k=string();ws();if(s.charAt(p++)!=':')throw new IllegalArgumentException("JSON colon");m.put(k,value());ws();c=s.charAt(p++);}while(c==',');return m;}
            if(c=='['){List<Object>a=new ArrayList<>();ws();if(s.charAt(p)==']'){p++;return a;}do{a.add(value());ws();c=s.charAt(p++);}while(c==',');return a;}
            if(c=='"'){p--;return string();}
            int a=p-1;while(p<s.length()&&",]} \r\n\t".indexOf(s.charAt(p))<0)p++;
            String v=s.substring(a,p);return switch(v){case "true"->true;case "false"->false;case "null"->null;default->Double.valueOf(v);};
        }
        String string(){if(s.charAt(p++)!='"')throw new IllegalArgumentException("JSON string");StringBuilder b=new StringBuilder();while(p<s.length()){char c=s.charAt(p++);if(c=='"')return b.toString();if(c=='\\'){c=s.charAt(p++);c=switch(c){case 'n'->'\n';case 'r'->'\r';case 't'->'\t';case 'b'->'\b';case 'f'->'\f';case 'u'->{char q=(char)Integer.parseInt(s.substring(p,p+4),16);p+=4;yield q;}default->c;};}b.append(c);}throw new IllegalArgumentException("JSON end");}
    }
    @SuppressWarnings("unchecked") static Map<String,Object> map(Object o){return o instanceof Map?(Map<String,Object>)o:Map.of();}
    @SuppressWarnings("unchecked") static List<Object> list(Object o){return o instanceof List?(List<Object>)o:List.of();}
    static double num(Object o,double d){return o instanceof Number n?n.doubleValue():d;}
    static String str(Object o,String d){return o==null?d:o.toString();}
    record V(double x,double y,double z){V add(V b){return new V(x+b.x,y+b.y,z+b.z);}V sub(V b){return new V(x-b.x,y-b.y,z-b.z);}V mul(double a){return new V(x*a,y*a,z*a);}double dot(V b){return x*b.x+y*b.y+z*b.z;}V cross(V b){return new V(y*b.z-z*b.y,z*b.x-x*b.z,x*b.y-y*b.x);}V unit(){return mul(1/Math.sqrt(dot(this)));}}
    static V vec(Object o){List<Object>a=list(o);return new V(num(a.get(0),0),num(a.get(1),0),num(a.get(2),0));}
    static V rotate(V v,String axis,double degrees){double a=Math.toRadians(degrees),s=Math.sin(a),c=Math.cos(a);return switch(axis){case "x"->new V(v.x,v.y*c-v.z*s,v.y*s+v.z*c);case "z"->new V(v.x*c-v.y*s,v.x*s+v.y*c,v.z);default->new V(v.x*c+v.z*s,v.y,-v.x*s+v.z*c);};}
    record Texture(int w,int h,int[] pixels){int sample(double u,double v){int x=Math.floorMod((int)Math.floor(u*w/16),w),y=Math.floorMod((int)Math.floor(v*h/16),h);return pixels[y*w+x];}}
    static int tintColor(int c,int t){return (c&0xff000000)|(((c>>16&255)*(t>>16&255)/255)<<16)|(((c>>8&255)*(t>>8&255)/255)<<8)|((c&255)*(t&255)/255);}
    static class Quad {
        final V p,u,v,n;final double iu,iv;final Texture texture;final double[]uv;final int rotation,tint;final boolean shade;
        Quad(V p,V u,V v,Texture t,double[]uv,int rotation,int tint,boolean shade){this.p=p;this.u=u;this.v=v;n=u.cross(v).unit();iu=1/u.dot(u);iv=1/v.dot(v);texture=t;this.uv=uv;this.rotation=rotation;this.tint=tint;this.shade=shade;}
        double hit(double ox,double oy,double oz,double dx,double dy,double dz){double den=n.x*dx+n.y*dy+n.z*dz;if(Math.abs(den)<1e-12)return Double.POSITIVE_INFINITY;double t=(n.x*(p.x-ox)+n.y*(p.y-oy)+n.z*(p.z-oz))/den;if(t<0)return Double.POSITIVE_INFINITY;double x=ox+dx*t-p.x,y=oy+dy*t-p.y,z=oz+dz*t-p.z;double a=(x*u.x+y*u.y+z*u.z)*iu,b=(x*v.x+y*v.y+z*v.z)*iv;return a>=-1e-9&&a<=1+1e-9&&b>=-1e-9&&b<=1+1e-9?t:Double.POSITIVE_INFINITY;}
        int color(double x,double y,double z){V r=new V(x-p.x,y-p.y,z-p.z);double a=r.dot(u)*iu,b=r.dot(v)*iv;for(int i=0;i<rotation/90;i++){double q=a;a=b;b=1-q;}int c=texture.sample(uv[0]+a*(uv[2]-uv[0]),uv[1]+b*(uv[3]-uv[1]));return tint<0?c:tintColor(c,tint);}
    }
    static final V CENTER=new V(.5,.5,.5);
    record Shape(Quad[] faces,boolean full,boolean opaque,int emission,double alpha,String id){}
    static class Assets implements AutoCloseable {
        final ZipFile zip;final Rules rules;
        final Map<String,Map<String,Object>>jsons=new HashMap<>(),models=new HashMap<>();
        final Map<String,Texture>textures=new HashMap<>();
        final SortedSet<String>fallbacks=new TreeSet<>(),missing=new TreeSet<>();
        final int grass,foliage,water;
        Assets(Path jar)throws Exception{zip=new ZipFile(jar.toFile());rules=new Rules(jar);Map<String,Object>plains=json("data/minecraft/worldgen/biome/plains.json"),effects=map(plains.get("effects"));
            if(!effects.containsKey("water_color"))throw new IOException("Jar plains biome has no water color");Object waterValue=effects.get("water_color");water=waterValue instanceof Number n?n.intValue():Integer.parseInt(waterValue.toString().replaceFirst("^#",""),16);
            double temp=num(plains.get("temperature"),.8),rain=num(plains.get("downfall"),.4);grass=biomeColor("grass",temp,rain);foliage=biomeColor("foliage",temp,rain);}
        int biomeColor(String name,double t,double r)throws IOException{BufferedImage im=ImageIO.read(zip.getInputStream(zip.getEntry("assets/minecraft/textures/colormap/"+name+".png")));return im.getRGB((int)((1-t)*255),(int)((1-r*t)*255))&0xffffff;}
        Map<String,Object>json(String path)throws IOException{if(jsons.containsKey(path))return jsons.get(path);ZipEntry e=zip.getEntry(path);Map<String,Object>m=e==null?Map.of():map(new Json(new String(zip.getInputStream(e).readAllBytes(),java.nio.charset.StandardCharsets.UTF_8)).value());jsons.put(path,m);return m;}
        String path(String id,String kind){return "assets/"+(id.contains(":")?id.substring(0,id.indexOf(':')):"minecraft")+"/"+kind+"/"+(id.contains(":")?id.substring(id.indexOf(':')+1):id)+".json";}
        Map<String,Object>model(String id)throws IOException{if(models.containsKey(id))return models.get(id);Map<String,Object>own=json(path(id,"models")),m=new LinkedHashMap<>();String parent=str(own.get("parent"),"");if(!parent.isEmpty())m.putAll(model(parent));Map<String,Object>ts=new LinkedHashMap<>(map(m.get("textures")));ts.putAll(map(own.get("textures")));boolean crossed=parent.contains("cross")||Boolean.TRUE.equals(m.get("_preview_cross"));m.putAll(own);m.put("_preview_cross",crossed);m.put("textures",ts);models.put(id,m);return m;}
        Texture texture(String id)throws IOException{if(textures.containsKey(id))return textures.get(id);String clean=id.replace("minecraft:","");String path="assets/minecraft/textures/"+clean+".png";ZipEntry e=zip.getEntry(path);Texture t;if(e==null){missing.add(path);t=new Texture(2,2,new int[]{0xffdc22dc,0xff292029,0xff292029,0xffdc22dc});}else{BufferedImage b=ImageIO.read(zip.getInputStream(e));int h=Math.min(b.getWidth(),b.getHeight());t=new Texture(b.getWidth(),h,b.getRGB(0,0,b.getWidth(),h,null,0,b.getWidth()));}textures.put(id,t);return t;}
        String resolve(Map<String,Object>ts,String s){for(int i=0;s.startsWith("#")&&i<20;i++){Object value=ts.get(s.substring(1));s=value instanceof Map?str(map(value).get("sprite"),"block/stone"):str(value,"block/stone");}return s;}
        Map<String,String>states(Block b){Map<String,String>s=new HashMap<>(Map.of("facing","north","half",b.id.endsWith("_door")?"lower":"bottom","shape","straight","type","bottom","axis","y","open","false","hanging","false","waterlogged","false","hinge","left","powered","false"));s.put("lit",b.id.contains("campfire")||b.id.contains("redstone_torch")?"true":"false");s.put("up",b.id.endsWith("_wall")?"true":"false");s.put("level","15");s.put("candles","1");s.put("pickles","1");s.put("charges","0");s.put("vault_state","inactive");s.put("trial_spawner_state","inactive");s.putAll(b.states);return s;}
        boolean condition(Object value,Map<String,String>s){Map<String,Object>m=map(value);if(m.containsKey("OR"))return list(m.get("OR")).stream().anyMatch(v->condition(v,s));if(m.containsKey("AND"))return list(m.get("AND")).stream().allMatch(v->condition(v,s));for(var e:m.entrySet())if(!Arrays.asList(e.getValue().toString().split("\\|")).contains(s.get(e.getKey())))return false;return true;}
        List<Map<String,Object>>applications(Block b,Map<String,String>s)throws IOException{Map<String,Object>def=json("assets/minecraft/blockstates/"+b.id+".json");List<Map<String,Object>>a=new ArrayList<>();Map<String,Object>vars=map(def.get("variants"));
            if(!vars.isEmpty()){String chosen=null;int best=-1;for(String key:vars.keySet()){boolean valid=true;int score=0;for(String p:key.isEmpty()?new String[0]:key.split(",")){String[]kv=p.split("=");if(b.states.containsKey(kv[0])&&!b.states.get(kv[0]).equals(kv[1]))valid=false;if(Objects.equals(s.get(kv[0]),kv[1]))score++;}if(valid&&score>best){best=score;chosen=key;}}if(chosen!=null){Object v=vars.get(chosen);a.add(map(v instanceof List?list(v).get(0):v));}}
            for(Object part:list(def.get("multipart"))){Map<String,Object>p=map(part);if(!p.containsKey("when")||condition(p.get("when"),s)){Object v=p.get("apply");a.add(map(v instanceof List?list(v).get(0):v));}}return a;}
        Shape shape(Block b,int connections)throws Exception{
            if(b.id.equals("air")||b.id.equals("cave_air")||b.id.equals("void_air"))return new Shape(new Quad[0],false,false,0,1,b.id);
            Map<String,String>s=states(b);String[]dirs={"north","east","south","west"};boolean wall=b.id.endsWith("_wall");for(int i=0;i<4;i++)if(!b.states.containsKey(dirs[i]))s.put(dirs[i],(connections&(1<<i))!=0?(wall?"low":"true"):(wall?"none":"false"));
            List<Quad>quads=new ArrayList<>();boolean full=false,crossed=false;double alpha=b.id.contains("glass")?.38:b.id.equals("water")?.48:1;
            if(b.id.equals("water")){addBox(quads,new V(0,0,0),new V(1,(connections&16)!=0?1:.875,1),texture("block/water_still"),water);}
            else {List<Map<String,Object>>apps=applications(b,s);for(Map<String,Object>app:apps){Map<String,Object>mod=model(str(app.get("model"),"")),ts=map(mod.get("textures"));
                crossed|=Boolean.TRUE.equals(mod.get("_preview_cross"));for(Object el:list(mod.get("elements"))){Map<String,Object>e=map(el);V from=vec(e.get("from")).mul(1/16.0),to=vec(e.get("to")).mul(1/16.0);if(from.equals(new V(0,0,0))&&to.equals(new V(1,1,1)))full=true;
                    for(var face:map(e.get("faces")).entrySet()){Map<String,Object>f=map(face.getValue());String tid=resolve(ts,str(f.get("texture"),"block/stone"));Texture tex=texture(tid);int tint=f.containsKey("tintindex")?(b.id.contains("leaves")?foliage:grass):-1;Quad q=face(from,to,face.getKey(),tex,f,tint,!Boolean.FALSE.equals(e.get("shade")));Map<String,Object>rot=map(e.get("rotation"));V p=q.p,u=q.p.add(q.u),v=q.p.add(q.v);
                        if(!rot.isEmpty()){V origin=vec(rot.get("origin")).mul(1/16.0);String axis=str(rot.get("axis"),"y");double angle=num(rot.get("angle"),0);boolean rescale=Boolean.TRUE.equals(rot.get("rescale"));p=elementRotate(p,origin,axis,angle,rescale);u=elementRotate(u,origin,axis,angle,rescale);v=elementRotate(v,origin,axis,angle,rescale);}
                        double rx=-num(app.get("x"),0),ry=-num(app.get("y"),0);p=variantRotate(p,rx,ry);u=variantRotate(u,rx,ry);v=variantRotate(v,rx,ry);quads.add(new Quad(p,u.sub(p),v.sub(p),tex,q.uv,q.rotation,tint,q.shade));}}
                }
                if(quads.isEmpty()){fallbacks.add(b.id);quads.addAll(fallbackCube(b,apps));}
            }
            if(!supported(b,full,crossed)){fallbacks.add(b.id);quads.clear();quads.addAll(fallbackCube(b,applications(b,s)));}Rules.Info info=rules.info(b,s);return new Shape(quads.toArray(Quad[]::new),full,info.occludes&&full,info.emission,alpha,b.id);
        }
        boolean supported(Block b,boolean full,boolean crossed){String id=b.id;return full||id.equals("water")||id.endsWith("_slab")||id.endsWith("_stairs")||id.endsWith("_fence")||id.endsWith("_wall")||id.endsWith("_bars")||id.equals("iron_bars")||id.endsWith("_pane")||id.endsWith("_door")||id.endsWith("_trapdoor")||id.endsWith("_carpet")||id.endsWith("_pressure_plate")||id.endsWith("lantern")||id.endsWith("chain")||id.endsWith("_sapling")||crossed;}
        List<Quad> fallbackCube(Block b,List<Map<String,Object>>apps)throws IOException{Map<String,Object>m=apps.isEmpty()?Map.of():model(str(apps.get(0).get("model"),"")),ts=map(m.get("textures"));String particle=resolve(ts,"#particle");List<Quad>q=new ArrayList<>();for(String d:List.of("up","down","north","south","east","west")){String key=d.equals("up")?"top":d.equals("down")?"bottom":"side";String tex=ts.containsKey(key)?resolve(ts,"#"+key):particle;for(String candidate:List.of("block/"+b.id+"_"+key,"block/"+b.id)){if(zip.getEntry("assets/minecraft/textures/"+candidate+".png")!=null){tex=candidate;break;}}q.add(face(new V(0,0,0),new V(1,1,1),d,texture(tex),Map.of(),-1,true));}return q;}        V elementRotate(V p,V o,String axis,double angle,boolean rescale){V q=rotate(p.sub(o),axis,angle);if(rescale){double r=1/Math.cos(Math.toRadians(angle));q=switch(axis){case "x"->new V(q.x,q.y*r,q.z*r);case "z"->new V(q.x*r,q.y*r,q.z);default->new V(q.x*r,q.y,q.z*r);};}return q.add(o);}
        V variantRotate(V p,double x,double y){return rotate(rotate(p.sub(CENTER),"x",x),"y",y).add(CENTER);}
        void addBox(List<Quad>a,V f,V t,Texture tex,int tint){for(String dir:List.of("up","down","north","south","east","west"))a.add(face(f,t,dir,tex,Map.of(),tint,true));}
        Quad face(V f,V t,String d,Texture tex,Map<String,Object>face,int tint,boolean shade){V p,u,v;double[]uv;
            switch(d){case "up"->{p=new V(f.x,t.y,f.z);u=new V(t.x-f.x,0,0);v=new V(0,0,t.z-f.z);uv=new double[]{f.x*16,f.z*16,t.x*16,t.z*16};}
                case "down"->{p=new V(f.x,f.y,t.z);u=new V(t.x-f.x,0,0);v=new V(0,0,f.z-t.z);uv=new double[]{f.x*16,16-t.z*16,t.x*16,16-f.z*16};}
                case "north"->{p=new V(t.x,t.y,f.z);u=new V(f.x-t.x,0,0);v=new V(0,f.y-t.y,0);uv=new double[]{16-t.x*16,16-t.y*16,16-f.x*16,16-f.y*16};}
                case "south"->{p=new V(f.x,t.y,t.z);u=new V(t.x-f.x,0,0);v=new V(0,f.y-t.y,0);uv=new double[]{f.x*16,16-t.y*16,t.x*16,16-f.y*16};}
                case "west"->{p=new V(f.x,t.y,f.z);u=new V(0,0,t.z-f.z);v=new V(0,f.y-t.y,0);uv=new double[]{f.z*16,16-t.y*16,t.z*16,16-f.y*16};}
                default->{p=new V(t.x,t.y,t.z);u=new V(0,0,f.z-t.z);v=new V(0,f.y-t.y,0);uv=new double[]{16-t.z*16,16-t.y*16,16-f.z*16,16-f.y*16};}}
            if(face.containsKey("uv")){List<Object>a=list(face.get("uv"));for(int i=0;i<4;i++)uv[i]=num(a.get(i),0);}return new Quad(p,u,v,tex,uv,(int)num(face.get("rotation"),0),tint,shade);}
        public void close()throws IOException{zip.close();}
    }
    /** Reads emission lambdas and Properties.noOcclusion/noCollision/ofLegacyCopy from jar bytecode.
     * javap is the rig JDK's disassembler; no Minecraft class is loaded or executed. */
    static class Rules {
        record Ins(int pc,String op,String arg,String comment){}
        record Definition(boolean occludes,String function,int captured,String source){}
        record Info(boolean occludes,int emission){}
        final Map<String,List<Ins>>methods=new HashMap<>();final Map<Integer,String>boot=new HashMap<>();
        final Map<String,Definition>definitions=new HashMap<>();final SortedMap<String,String>used=new TreeMap<>();
        final Path jar;
        Rules(Path jar)throws Exception{this.jar=jar;
            String code=disassemble("net.minecraft.world.level.block.Blocks");parse(code);
            List<Ins>segment=new ArrayList<>();for(Ins in:methods.get("<clinit>")){segment.add(in);Matcher m=Pattern.compile("Field ([A-Z0-9_]+):Lnet/minecraft/world/level/block/").matcher(in.comment);if(in.op.equals("putstatic")&&m.find()){definitions.put(m.group(1).toLowerCase(Locale.ROOT),definition(segment));segment=new ArrayList<>();}}
        }
        String disassemble(String cls)throws Exception{String exe=Path.of(System.getProperty("java.home"),"bin",System.getProperty("os.name").startsWith("Windows")?"javap.exe":"javap").toString();Process p=new ProcessBuilder(exe,"-p","-c","-v","-classpath",jar.toString(),cls).redirectErrorStream(true).start();String text=new String(p.getInputStream().readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);if(p.waitFor()!=0)throw new IOException("javap failed: "+text);return text;}
        void parse(String text){String method=null;Integer bootstrap=null;boolean boots=false;
            Pattern instruction=Pattern.compile("\\s+(\\d+):\\s+(\\w+)(.*)");
            for(String line:text.split("\\R")){
                if(line.equals("BootstrapMethods:")){boots=true;continue;}
                if(boots){Matcher bm=Pattern.compile("  (\\d+): #").matcher(line);if(bm.find())bootstrap=Integer.parseInt(bm.group(1));Matcher target=Pattern.compile("REF_invokeStatic ([a-zA-Z0-9_/$]+)\\.([^:]+):").matcher(line);if(target.find()&&bootstrap!=null)boot.put(bootstrap,target.group(1).equals("net/minecraft/world/level/block/Blocks")?target.group(2):target.group(1)+"."+target.group(2));continue;}
                if(line.equals("  static {};")){method="<clinit>";methods.put(method,new ArrayList<>());continue;}
                if(line.startsWith("  ")&&!line.startsWith("   ")&&line.contains("(")&&line.endsWith(";")){String start=line.substring(0,line.indexOf('('));method=start.substring(start.lastIndexOf(' ')+1);methods.put(method,new ArrayList<>());continue;}
                Matcher m=instruction.matcher(line);if(method!=null&&m.matches()){String tail=m.group(3).strip();int ci=tail.indexOf("//");methods.get(method).add(new Ins(Integer.parseInt(m.group(1)),m.group(2),ci<0?tail:tail.substring(0,ci).strip(),ci<0?"":tail.substring(ci+2).strip()));}
            }
        }
        Integer constant(Ins in){if(in.op.startsWith("iconst_"))return in.op.equals("iconst_m1")?-1:Integer.parseInt(in.op.substring(7));if(in.op.equals("bipush")||in.op.equals("sipush"))return Integer.parseInt(in.arg);if(in.comment.startsWith("int "))return Integer.parseInt(in.comment.substring(4));return null;}
        Definition definition(List<Ins>code){boolean occ=true;String fn=null,source="Blocks.<clinit> default Properties";int captured=0;String ref=null;
            for(int i=0;i<code.size();i++){Ins in=code.get(i);Matcher r=Pattern.compile("Field ([A-Z0-9_]+):Lnet/minecraft/world/level/block/").matcher(in.comment);if(in.op.equals("getstatic")&&r.find())ref=r.group(1).toLowerCase(Locale.ROOT);
                if(in.op.equals("invokestatic")&&in.comment.matches("Method [A-Za-z0-9_$]+:.*\\)Lnet/minecraft/world/level/block/state/BlockBehaviour\\$Properties;")){String helper=in.comment.substring(7,in.comment.indexOf(':'));if(methods.containsKey(helper)){Definition h=definition(methods.get(helper));occ=h.occludes;fn=h.function;captured=h.captured;source+="; helper "+helper+"; "+h.source;}}
                if(in.op.equals("invokedynamic")){Matcher bm=Pattern.compile("InvokeDynamic #(\\d+):").matcher(in.comment);if(bm.find()){String helper=boot.get(Integer.parseInt(bm.group(1)));List<Ins>hc=methods.get(helper);if(hc!=null&&hc.stream().anyMatch(q->q.comment.contains("Properties.of:"))){Definition h=definition(hc);occ=h.occludes;fn=h.function;captured=h.captured;source+="; helper "+helper+"; "+h.source;}}}
                if(in.comment.contains("Properties.ofLegacyCopy:")||in.comment.contains("Properties.ofFullCopy:")){Definition d=definitions.get(ref);if(d!=null){occ=d.occludes;fn=d.function;captured=d.captured;source="Blocks.<clinit> copy "+ref+"; "+d.source;}}
                if(in.comment.contains("Properties.noOcclusion:")||in.comment.contains("Properties.noCollision:")){occ=false;source+="; "+in.comment.split(":")[0];}
                if(in.comment.contains("Properties.lightLevel:")){Ins call=code.get(i-1);if(call.op.equals("invokedynamic")){Matcher m=Pattern.compile("InvokeDynamic #(\\d+):").matcher(call.comment);if(!m.find())throw new IllegalArgumentException("Light bootstrap missing");fn=boot.get(Integer.parseInt(m.group(1)));captured=0;}
                    else if(call.comment.contains("litBlockEmission:")){fn="lambda$litBlockEmission$0";captured=Objects.requireNonNull(constant(code.get(i-2)));}
                    else {fn=(call.op.equals("getstatic")?"field:":"external:")+call.comment;captured=i>1&&constant(code.get(i-2))!=null?constant(code.get(i-2)):0;}source+="; Blocks."+fn+"("+captured+")";}
            }return new Definition(occ,fn,captured,source);
        }
        int capturedLevel(Block block,Definition d)throws IOException{
            for(var method:new ArrayList<>(methods.entrySet())){
                List<Ins>body=method.getValue();if(!d.source.contains(method.getKey()+";")||body.stream().noneMatch(i->i.op.equals("tableswitch")))continue;
                Ins switchField=body.stream().filter(i->i.op.equals("getstatic")&&i.comment.contains("$SwitchMap$")).findFirst().orElseThrow();
                String ref=switchField.comment.substring(6,switchField.comment.indexOf(':')),cls=ref.substring(0,ref.lastIndexOf('.'));loadClass(cls);
                Map<String,Integer>selectors=new LinkedHashMap<>();String enumName=null;
                for(Ins in:methods.get(cls+".<clinit>")){if(in.op.equals("getstatic")&&in.comment.contains("$WeatherState."))enumName=property(in.comment.substring(6));Integer c=constant(in);if(c!=null&&enumName!=null)selectors.put(enumName,c);if(in.op.equals("iastore"))enumName=null;}
                String id=block.id.replaceFirst("^waxed_","");Integer selector=selectors.values().stream().findFirst().orElseThrow();for(var e:selectors.entrySet())if(id.startsWith(e.getKey()+"_"))selector=e.getValue();
                int chosen=selector;Ins branch=body.stream().filter(i->i.pc==chosen&&i.op.matches("[0-9]+")).findFirst().orElseThrow();int pc=Integer.parseInt(branch.op);Ins value=body.stream().filter(i->i.pc==pc&&constant(i)!=null).findFirst().orElseThrow();return constant(value);
            }return d.captured;
        }        Info info(Block b,Map<String,String>states)throws Exception{Definition d=definitions.get(b.id);if(d==null){String base=b.id.replaceFirst("^waxed_", "").replaceFirst("^(exposed|weathered|oxidized)_", "");d=definitions.get(base);if(d==null){base=base.replaceFirst("^(light_blue|light_gray|white|orange|magenta|yellow|lime|pink|gray|cyan|purple|blue|brown|green|red|black)_", "");d=definitions.get(base);if(d==null)d=definitions.get("dyed_"+base);}}
            if(d==null)throw new IOException("No jar property definition for "+b.id+"; refusing to guess emission/opacity");
            Map<String,String>values=new HashMap<>(states);values.put("charge",states.getOrDefault("charges","0"));values.put("state",states.getOrDefault(b.id.equals("vault")?"vault_state":"trial_spawner_state","inactive"));values.put("phase",states.getOrDefault("sculk_sensor_phase","inactive"));int e=d.function==null?0:evaluate(d.function,capturedLevel(b,d),values);if(e<0||e>15)throw new IOException("Emission outside 0..15 for "+b.id+": "+e);used.put(b.id,"emission="+e+", canOcclude="+d.occludes+"; "+d.source);return new Info(d.occludes,e);
        }
        String property(String field){String key=field.substring(field.lastIndexOf('.')+1).split(":")[0].toLowerCase(Locale.ROOT);return key.equals("state")?"state":key;}
        int asInt(Object a){if(a instanceof Number n)return n.intValue();return a.toString().equals("true")?1:a.toString().equals("false")?0:Integer.parseInt(a.toString());}
        final Map<String,Map<Integer,String>>externalBoots=new HashMap<>();
        void loadClass(String cls)throws IOException{
            if(externalBoots.containsKey(cls))return;
            Map<String,List<Ins>>saved=new HashMap<>(methods);Map<Integer,String>savedBoot=new HashMap<>(boot);
            Map<String,List<Ins>>loaded=null;Map<Integer,String>loadedBoot=null;
            methods.clear();boot.clear();
            try{parse(disassemble(cls.replace('/','.')));loaded=new HashMap<>(methods);loadedBoot=new HashMap<>(boot);}
            catch(Exception ex){throw new IOException(ex);}
            finally{methods.clear();methods.putAll(saved);boot.clear();boot.putAll(savedBoot);}
            externalBoots.put(cls,loadedBoot);for(var e:loaded.entrySet())methods.put(cls+"."+e.getKey(),e.getValue());
        }        // Enum light values are constructor arguments in the jar, not a hand-maintained table.
        int enumLight(String cls,String value)throws IOException{
            loadClass(cls);String serialized=null,reference=null;Integer level=null;
            for(Ins in:methods.get(cls+".<clinit>")){
                if(in.op.equals("new")){serialized=null;reference=null;level=null;}
                if(in.comment.startsWith("String ")){serialized=in.comment.substring(7);level=null;}
                Integer c=constant(in);if(c!=null&&serialized!=null&&serialized.equals(serialized.toLowerCase(Locale.ROOT))&&level==null)level=c;
                if(in.op.equals("getstatic")&&in.comment.contains("$LightLevel."))reference=in.comment.substring(6,in.comment.indexOf(':'));
                if(in.op.equals("putstatic")&&value.equals(serialized)){
                    if(level!=null)return level;
                    if(reference!=null){String type=reference.substring(0,reference.lastIndexOf('.')),member=reference.substring(reference.lastIndexOf('.')+1);loadClass(type);Integer result=null;
                        for(Ins q:methods.get(type+".<clinit>")){if(q.op.equals("new"))result=null;Integer n=constant(q);if(n!=null)result=n;if(q.op.equals("putstatic")&&q.comment.startsWith("Field "+member+":")){if(result!=null)return result;}}
                    }
                }
            }throw new IOException("No jar enum light value for "+cls+"."+value);
        }        int evaluate(String name,int captured,Map<String,String>state)throws IOException{
            if(name.startsWith("external:")){Matcher factory=Pattern.compile("(?:InterfaceMethod|Method) ([a-zA-Z0-9_/$]+)\\.([^:]+):").matcher(name);if(!factory.find())throw new IOException("Emission factory "+name);String cls=factory.group(1);loadClass(cls);for(Ins in:methods.get(cls+"."+factory.group(2))){Matcher bm=Pattern.compile("InvokeDynamic #(\\d+):").matcher(in.comment);if(bm.find())return evaluate(externalBoots.get(cls).get(Integer.parseInt(bm.group(1))),captured,state);}throw new IOException("No lambda in emission factory "+name);}
            if(name.startsWith("field:")){String field=name.substring(6).replace("Field ","");String cls=field.substring(0,field.lastIndexOf('.')),member=field.substring(field.lastIndexOf('.')+1,field.indexOf(':'));loadClass(cls);List<Ins>init=methods.get(cls+".<clinit>");for(int i=1;i<init.size();i++){Ins in=init.get(i);if(in.op.equals("putstatic")&&in.comment.contains("Field "+member+":")){Ins prev=init.get(i-1);Matcher bm=Pattern.compile("InvokeDynamic #(\\d+):").matcher(prev.comment);if(bm.find())return evaluate(externalBoots.get(cls).get(Integer.parseInt(bm.group(1))),captured,state);}}throw new IOException("No emission field initializer: "+name);}
            if(name.contains("/"))loadClass(name.substring(0,name.lastIndexOf('.')));
            List<Ins>code=methods.get(name);if(name.endsWith("MultifaceBlock.hasAnyFace")&&code!=null&&code.stream().anyMatch(q->q.comment.contains("hasFace:"))){for(String direction:List.of("north","south","east","west","up","down"))if(state.getOrDefault(direction,"false").equals("true"))return 1;return 0;}if(code==null)throw new IOException("Emission function absent: "+name);Map<Integer,Integer>pcs=new HashMap<>();for(int i=0;i<code.size();i++)pcs.put(code.get(i).pc,i);List<Object>stack=new ArrayList<>();
            for(int ip=0,steps=0;ip<code.size()&&steps++<500;ip++){Ins in=code.get(ip);String op=in.op;Integer c=constant(in);if(c!=null){stack.add(c);continue;}
                if(in.comment.startsWith("float ")){stack.add(Float.parseFloat(in.comment.substring(6).replace("f","")));continue;}if(op.equals("i2f")){stack.add((float)asInt(stack.remove(stack.size()-1)));continue;}if(List.of("fmul","fdiv","fadd","fsub").contains(op)){float a=((Number)stack.remove(stack.size()-1)).floatValue(),b=((Number)stack.remove(stack.size()-1)).floatValue();stack.add(switch(op){case "fmul"->b*a;case "fdiv"->b/a;case "fadd"->b+a;default->b-a;});continue;}if(op.startsWith("aload")){stack.add(state);continue;}if(op.startsWith("iload")){stack.add(captured);continue;}if(op.equals("checkcast"))continue;
                if(op.equals("getstatic")){stack.add(property(in.comment.substring("Field ".length())));continue;}
                if(op.equals("ireturn"))return asInt(stack.remove(stack.size()-1));
                if(op.equals("goto")){ip=pcs.get(Integer.parseInt(in.arg))-1;continue;}
                if(op.startsWith("if")){Object a=stack.remove(stack.size()-1);boolean jump;
                    if(op.startsWith("if_icmp")||op.startsWith("if_acmp")){Object b=stack.remove(stack.size()-1);String test=op.substring(7);jump=switch(test){case "eq"->b.equals(a);case "ne"->!b.equals(a);case "gt"->asInt(b)>asInt(a);case "ge"->asInt(b)>=asInt(a);case "lt"->asInt(b)<asInt(a);case "le"->asInt(b)<=asInt(a);default->throw new IOException("Emission branch "+op);};}
                    else jump=switch(op){case "ifeq"->asInt(a)==0;case "ifne"->asInt(a)!=0;case "ifgt"->asInt(a)>0;case "ifge"->asInt(a)>=0;case "iflt"->asInt(a)<0;case "ifle"->asInt(a)<=0;default->throw new IOException("Emission branch "+op);};if(jump)ip=pcs.get(Integer.parseInt(in.arg))-1;continue;}
                if(List.of("iadd","isub","imul","idiv","ishl","iand","ior").contains(op)){int a=asInt(stack.remove(stack.size()-1)),b=asInt(stack.remove(stack.size()-1));stack.add(switch(op){case "iadd"->b+a;case "isub"->b-a;case "imul"->b*a;case "idiv"->b/a;case "ishl"->b<<a;case "iand"->b&a;default->b|a;});continue;}
                if(op.startsWith("invoke")){String call=in.comment;
                    if(call.contains(".getValue:")){String prop=stack.remove(stack.size()-1).toString();stack.remove(stack.size()-1);stack.add(state.getOrDefault(prop,"false"));continue;}
                                        if(call.contains("net/minecraft/util/Mth.floor:(F)I")){float v=((Number)stack.remove(stack.size()-1)).floatValue();stack.add((int)Math.floor(v));continue;}
                    if(call.contains(".lightLevel:()I")){Matcher owner=Pattern.compile("Method ([a-zA-Z0-9_/$]+)\\.lightLevel:").matcher(call);if(owner.find()){String value=stack.remove(stack.size()-1).toString();stack.add(enumLight(owner.group(1),value));continue;}}
                    if(call.contains("Boolean.booleanValue:")||call.contains("Integer.intValue:")){Object v=stack.remove(stack.size()-1);stack.add(asInt(v));continue;}
                    Matcher mc=Pattern.compile("Method ([a-zA-Z0-9_/$]+)\\.([^:]+):\\((.*?)\\)([IZ])").matcher(call);
                    if(in.op.equals("invokestatic")&&mc.find()){String descriptor=mc.group(3);int count=0;for(int k=0;k<descriptor.length();k++){count++;if(descriptor.charAt(k)=='L')k=descriptor.indexOf(';',k);}Object[]params=new Object[count];for(int k=count-1;k>=0;k--)params[k]=stack.remove(stack.size()-1);int cap=0;for(Object param:params)if(!(param instanceof Map))cap=asInt(param);stack.add(evaluate(mc.group(1)+"."+mc.group(2),cap,state));continue;}
                }
                throw new IOException("Unsupported jar emission instruction in "+name+": "+in+" (refusing to guess)");
            }throw new IOException("Emission function did not return: "+name);
        }
    }
    static class Scene {
        final World world;final Assets assets;final Shape[]shapes;final int[]shapeIds;final Light light;
        final int[]dx={0,1,0,-1},dz={-1,0,1,0};
        Scene(World w,Assets a)throws Exception{world=w;assets=a;List<Shape>ss=new ArrayList<>();Map<Long,Integer>cache=new HashMap<>();
            for(int i=0;i<w.palette.size();i++){ss.add(a.shape(w.palette.get(i),0));cache.put((long)i<<5,i);}
            shapeIds=new int[w.cells.length];for(int y=w.y0;y<=w.y1;y++)for(int z=w.z0;z<=w.z1;z++)for(int x=w.x0;x<=w.x1;x++){int pi=w.cells[w.index(x,y,z)];Block b=w.palette.get(pi);int mask=b.id.equals("water")&&w.get(x,y+1,z).id.equals("water")?16:0;
                if(connected(b.id))for(int d=0;d<4;d++){Block n=w.get(x+dx[d],y,z+dz[d]);Integer ni=w.ids.get(n);boolean solid=ni!=null?ss.get(ni).full&&!n.id.contains("leaves")&&!n.id.contains("glass"):n.equals(GRASS)||n.equals(DIRT);if(solid||n.id.equals(b.id)||(b.id.contains("pane")||b.id.equals("iron_bars"))&&(n.id.contains("pane")||n.id.equals("iron_bars")))mask|=1<<d;}
                long key=((long)pi<<5)|mask;Integer si=cache.get(key);if(si==null){si=ss.size();ss.add(a.shape(b,mask));cache.put(key,si);}shapeIds[w.index(x,y,z)]=si;
            }shapes=ss.toArray(Shape[]::new);light=new Light(this);
        }
        static boolean connected(String id){return id.endsWith("_fence")||id.endsWith("_wall")||id.endsWith("_pane")||id.endsWith("_bars");}
        int shapeIndex(int x,int y,int z){return world.inside(x,y,z)?shapeIds[world.index(x,y,z)]:y < -61?2:y == -61?1:0;}
        Shape get(int x,int y,int z){return shapes[shapeIndex(x,y,z)];}
    }
    static class Light {
        final int x0,y0,z0,nx,ny,nz;final byte[]values;
        Light(Scene scene){World w=scene.world;x0=w.x0-15;y0=w.y0-15;z0=w.z0-15;nx=w.nx+30;ny=w.ny+30;nz=w.nz+30;values=new byte[Math.multiplyExact(Math.multiplyExact(nx,ny),nz)];
            @SuppressWarnings("unchecked") ArrayDeque<Integer>[]q=new ArrayDeque[16];for(int i=0;i<16;i++)q[i]=new ArrayDeque<>();
            for(int y=w.y0;y<=w.y1;y++)for(int z=w.z0;z<=w.z1;z++)for(int x=w.x0;x<=w.x1;x++){int e=scene.get(x,y,z).emission;if(e>0){int i=index(x,y,z);values[i]=(byte)e;q[e].add(i);}}
            int[]dx={1,-1,0,0,0,0},dy={0,0,1,-1,0,0},dz={0,0,0,0,1,-1};
            for(int level=15;level>1;level--)while(!q[level].isEmpty()){int i=q[level].removeFirst();if(values[i]!=level)continue;int x=i%nx+x0,z=i/nx%nz+z0,y=i/(nx*nz)+y0;for(int d=0;d<6;d++){int xx=x+dx[d],yy=y+dy[d],zz=z+dz[d];if(!inside(xx,yy,zz)||scene.get(xx,yy,zz).opaque)continue;int j=index(xx,yy,zz);if(values[j]<level-1){values[j]=(byte)(level-1);q[level-1].add(j);}}}
        }
        boolean inside(int x,int y,int z){return x>=x0&&x<x0+nx&&y>=y0&&y<y0+ny&&z>=z0&&z<z0+nz;}
        int index(int x,int y,int z){return ((y-y0)*nz+z-z0)*nx+x-x0;}
        int get(int x,int y,int z){return inside(x,y,z)?values[index(x,y,z)]:0;}
    }
    record View(String name,int width,int height,V origin,V forward,V right,V up,double scale,double tangent,boolean night,boolean perspective,double[]clip){}
    static View view(String line,World w){String[]p=line.strip().split("\\s+");V center=new V((w.x0+w.x1+1)/2.0,(w.y0+w.y1+1)/2.0,(w.z0+w.z1+1)/2.0);double[]clip={w.x0,w.y0,w.z0,w.x1+1,w.y1+1,w.z1+1};V f,r,u;boolean night=false;String name;int width,height;double scale=8;
        switch(p[0]){
            case "iso"->{if(p.length!=3||!List.of("NE","NW","SE","SW").contains(p[1]))throw new IllegalArgumentException("iso corner day|night");double x=p[1].contains("E")?1:-1,z=p[1].contains("S")?1:-1;f=new V(-x,-1,-z).unit();night=p[2].equals("night");name="iso-"+p[1]+"-"+p[2];}
            case "plan"->{if(p.length!=2)throw new IllegalArgumentException("plan top|y");f=new V(0,-1,0);if(!p[1].equals("top"))clip[4]=Math.min(clip[4],Double.parseDouble(p[1])+1);name="plan-"+p[1];}
            case "elev"->{if(p.length!=2)throw new IllegalArgumentException("elev N|E|S|W");f=switch(p[1]){case "N"->new V(0,0,1);case "S"->new V(0,0,-1);case "E"->new V(-1,0,0);case "W"->new V(1,0,0);default->throw new IllegalArgumentException("Bad elevation");};name="elev-"+p[1];}
            case "section"->{if(p.length!=3)throw new IllegalArgumentException("section x|z value");double at=Double.parseDouble(p[2]);if(p[1].equals("x")){clip[0]=Math.max(clip[0],at);clip[3]=Math.min(clip[3],at+1);f=new V(-1,0,0);}else if(p[1].equals("z")){clip[2]=Math.max(clip[2],at);clip[5]=Math.min(clip[5],at+1);f=new V(0,0,1);}else throw new IllegalArgumentException("Section axis");name="section-"+p[1]+"-"+p[2];}
            default->throw new IllegalArgumentException("Unknown view: "+line);
        }
        if(clip[0]>=clip[3]||clip[1]>=clip[4]||clip[2]>=clip[5])throw new IllegalArgumentException("Cut misses box: "+line);
        r=Math.abs(f.y)>.999?new V(1,0,0):f.cross(new V(0,1,0)).unit();u=r.cross(f).unit();
        double minR=Double.POSITIVE_INFINITY,maxR=-minR,minU=minR,maxU=-minR,minF=minR;
        for(int i=0;i<8;i++){V v=new V(clip[(i&1)==0?0:3],clip[(i&2)==0?1:4],clip[(i&4)==0?2:5]);double rr=v.dot(r),uu=v.dot(u);minR=Math.min(minR,rr);maxR=Math.max(maxR,rr);minU=Math.min(minU,uu);maxU=Math.max(maxU,uu);minF=Math.min(minF,v.dot(f));}
        int margin=p[0].equals("plan")||p[0].equals("section")||p[0].equals("elev")?0:16;width=(int)Math.ceil((maxR-minR)*scale)+2*margin;height=(int)Math.ceil((maxU-minU)*scale)+2*margin;
        V origin=r.mul((minR+maxR)/2).add(u.mul((minU+maxU)/2)).add(f.mul(minF-2));return new View(name,width,height,origin,f,r,u,scale,0,night,false,clip);
    }
    static View parseView(String line,World w){String[]p=line.strip().split("\\s+");if(p[0].equals("iso")&&(p.length!=3||!Set.of("day","night").contains(p[2])))throw new IllegalArgumentException("iso needs day or night");if(p[0].equals("eye")&&(p.length!=9||!Set.of("day","night").contains(p[7])))throw new IllegalArgumentException("eye needs day or night");if(!p[0].equals("eye"))return view(line,w);if(p.length!=9)throw new IllegalArgumentException("eye x y z yaw pitch fov day|night name");double yaw=Math.toRadians(Double.parseDouble(p[4])),pitch=Math.toRadians(Double.parseDouble(p[5])),fov=Double.parseDouble(p[6]);if(!(fov>0&&fov<180))throw new IllegalArgumentException("FOV outside 0..180");V f=new V(-Math.sin(yaw)*Math.cos(pitch),-Math.sin(pitch),Math.cos(yaw)*Math.cos(pitch)),r=new V(-Math.cos(yaw),0,-Math.sin(yaw)),u=r.cross(f);return new View("eye-"+p[8],1280,720,new V(Double.parseDouble(p[1]),Double.parseDouble(p[2])+1.62,Double.parseDouble(p[3])),f,r,u,0,Math.tan(Math.toRadians(fov)/2),p[7].equals("night"),true,new double[]{w.x0,w.y0,w.z0,w.x1+1,w.y1+1,w.z1+1});}
    static int rgb(double r,double g,double b){return 0xff000000|((int)Math.max(0,Math.min(255,r))<<16)|((int)Math.max(0,Math.min(255,g))<<8)|(int)Math.max(0,Math.min(255,b));}
    static int sky(boolean night,double dy){double t=Math.max(0,Math.min(1,dy*.8+.3));return night?rgb(10-5*t,15-6*t,29-11*t):rgb(183-73*t,210-45*t,235+15*t);}
    static class Tracer {
        final Scene scene;final View view;
        Tracer(Scene scene,View view){this.scene=scene;this.view=view;}
        int ground(double ox,double oy,double oz,double dx,double dy,double dz,int background){if(dy>=0)return background;double t=(-60-oy)/dy;if(t<0)return background;double x=ox+dx*t,z=oz+dz*t;Shape g=scene.shapes[1];Quad top=Arrays.stream(g.faces).max(Comparator.comparingDouble(q->q.p.y)).orElseThrow();int c=top.color(x-Math.floor(x),1,z-Math.floor(z));int light=scene.light.get((int)Math.floor(x),-60,(int)Math.floor(z));double brightness=view.night?.035+.965*Math.pow(light/15.0,2):1;return shade(c,brightness,brightness*.91,brightness*.77,t,background);}
        int shade(int color,double br,double bg,double bb,double distance,int fog){double haze=view.perspective?1-Math.exp(-distance/(view.night?150:240)):0;return rgb((color>>16&255)*br*(1-haze)+(fog>>16&255)*haze,(color>>8&255)*bg*(1-haze)+(fog>>8&255)*haze,(color&255)*bb*(1-haze)+(fog&255)*haze);}
        int ray(double ox,double oy,double oz,double dx,double dy,double dz){int fog=sky(view.night,dy),background=view.perspective?ground(ox,oy,oz,dx,dy,dz,fog):fog;double enter=0,end=2000;double[]b=view.clip;
            if(dx==0){if(ox<b[0]||ox>b[3])return background;}else{double a=(b[0]-ox)/dx,c=(b[3]-ox)/dx;enter=Math.max(enter,Math.min(a,c));end=Math.min(end,Math.max(a,c));}
            if(dy==0){if(oy<b[1]||oy>b[4])return background;}else{double a=(b[1]-oy)/dy,c=(b[4]-oy)/dy;enter=Math.max(enter,Math.min(a,c));end=Math.min(end,Math.max(a,c));}
            if(dz==0){if(oz<b[2]||oz>b[5])return background;}else{double a=(b[2]-oz)/dz,c=(b[5]-oz)/dz;enter=Math.max(enter,Math.min(a,c));end=Math.min(end,Math.max(a,c));}
            if(end<enter)return background;
            // If commands carved away the ground plane inside the box, the implicit
            // outside world starts at the exit boundary, not at the removed grass.
            if(view.perspective&&dy<0){double gt=(-60-oy)/dy;if(gt>=enter&&gt<=end){double ex=ox+dx*(end+1e-6),ey=oy+dy*(end+1e-6),ez=oz+dz*(end+1e-6);int bx=(int)Math.floor(ex),by=(int)Math.floor(ey),bz=(int)Math.floor(ez);Shape ground=scene.get(bx,by,bz);double nearest=Double.POSITIVE_INFINITY;Quad face=null;for(Quad q:ground.faces){double hit=q.hit(ox-bx,oy-by,oz-bz,dx,dy,dz);if(hit>=end-1e-5&&hit<nearest){face=q;nearest=hit;}}if(face!=null){int color=face.color(ox+dx*nearest-bx,oy+dy*nearest-by,oz+dz*nearest-bz);double brightness=view.night?.035:1;background=shade(color,brightness,brightness,brightness,nearest,fog);}else background=fog;}}
            // A superflat surface before the box occludes it; the rest is handled by voxel traversal.
            if(view.perspective&&dy<0){double gt=(-60-oy)/dy;if(gt>0&&gt<enter){double gx=ox+dx*gt,gz=oz+dz*gt;if(gx<b[0]||gx>=b[3]||gz<b[2]||gz>=b[5])return background;}}
            double t=enter+1e-7;int x=(int)Math.floor(ox+dx*t),y=(int)Math.floor(oy+dy*t),z=(int)Math.floor(oz+dz*t);int sx=dx>0?1:-1,sy=dy>0?1:-1,sz=dz>0?1:-1;
            double tx=dx==0?Double.POSITIVE_INFINITY:(x+(sx>0?1:0)-ox)/dx,ty=dy==0?Double.POSITIVE_INFINITY:(y+(sy>0?1:0)-oy)/dy,tz=dz==0?Double.POSITIVE_INFINITY:(z+(sz>0?1:0)-oz)/dz;
            double ix=Math.abs(1/dx),iy=Math.abs(1/dy),iz=Math.abs(1/dz);double remain=1,rr=0,gg=0,bb=0;
            while(t<=end+1e-7&&remain>.015){double exit=Math.min(end,Math.min(tx,Math.min(ty,tz)));Shape shape=scene.get(x,y,z);double last=t-1e-6;
                for(int layer=0;layer<8&&shape.faces.length>0&&remain>.015;layer++){double nearest=Double.POSITIVE_INFINITY;Quad face=null;int color=0;
                    for(Quad q:shape.faces){double h=q.hit(ox-x,oy-y,oz-z,dx,dy,dz);if(h<last||h>exit+1e-7||h>nearest+1e-9)continue;int col=q.color(ox+dx*h-x,oy+dy*h-y,oz+dz*h-z);if((col>>>24)<24)continue;nearest=h;face=q;color=col;}
                    if(face==null)break;last=nearest+1e-6;V n=face.n;double dot=n.x*dx+n.y*dy+n.z*dz;if(dot>0)n=n.mul(-1);double hx=ox+dx*nearest,hy=oy+dy*nearest,hz=oz+dz*nearest;
                    int ax=(int)Math.floor(hx+n.x*1e-5),ay=(int)Math.floor(hy+n.y*1e-5),az=(int)Math.floor(hz+n.z*1e-5);
                    if(shape.alpha<1&&scene.get(ax,ay,az).id.equals(shape.id)&&!(ax==x&&ay==y&&az==z))continue;
                    double faceShade=!face.shade?1:Math.abs(n.y)>.9?(n.y>0?1:.5):.6*Math.abs(n.x)+.8*Math.abs(n.z);
                    int light=Math.max(shape.emission,Math.max(scene.light.get(ax,ay,az),scene.light.get(x,y,z)));double block=Math.pow(light/15.0,2);double brightness=view.night?.035+.965*block:1;
                    double emission=shape.emission>0?shape.emission/15.0:0;double br=Math.max(emission,brightness*faceShade),bg=Math.max(emission,brightness*faceShade*(view.night?.86:1)),bl=Math.max(emission,brightness*faceShade*(view.night?.65:1));
                    int c=shade(color,br,bg,bl,nearest,fog);double alpha=shape.alpha*((color>>>24)/255.0);rr+=remain*alpha*(c>>16&255);gg+=remain*alpha*(c>>8&255);bb+=remain*alpha*(c&255);remain*=1-alpha;
                }
                if(tx<=ty&&tx<=tz){x+=sx;t=tx;tx+=ix;}else if(ty<=tz){y+=sy;t=ty;ty+=iy;}else{z+=sz;t=tz;tz+=iz;}
                t+=1e-8;
            }
            return rgb(rr+remain*(background>>16&255),gg+remain*(background>>8&255),bb+remain*(background&255));
        }
        BufferedImage render(){BufferedImage image=new BufferedImage(view.width,view.height,BufferedImage.TYPE_INT_RGB);int[]pixels=((java.awt.image.DataBufferInt)image.getRaster().getDataBuffer()).getData();
            IntStream.range(0,view.height).parallel().forEach(y->{for(int x=0;x<view.width;x++){V o=view.origin,d=view.forward;if(view.perspective){double a=(2*(x+.5)/view.width-1)*view.tangent*view.width/view.height,c=(1-2*(y+.5)/view.height)*view.tangent;d=d.add(view.right.mul(a)).add(view.up.mul(c)).unit();}else{o=o.add(view.right.mul((x+.5-view.width/2.0)/view.scale)).add(view.up.mul((view.height/2.0-y-.5)/view.scale));}pixels[y*view.width+x]=ray(o.x,o.y,o.z,d.x,d.y,d.z);}});return image;
        }
    }
    static String sha(Path p)throws Exception{MessageDigest d=MessageDigest.getInstance("SHA-256");try(InputStream in=Files.newInputStream(p)){byte[]buf=new byte[65536];for(int n;(n=in.read(buf))>=0;)d.update(buf,0,n);}return HexFormat.of().formatHex(d.digest());}
    static List<String>lines(Path p)throws IOException{return Files.readAllLines(p).stream().map(String::strip).filter(s->!s.isEmpty()&&!s.startsWith("#")).toList();}
    static void renderAll(Scene scene,Path views,Path out)throws Exception{Files.createDirectories(out);List<String>index=new ArrayList<>(),record=new ArrayList<>();Set<String>names=new HashSet<>();long total=System.nanoTime();for(String line:lines(views)){View v=parseView(line,scene.world);if(!v.name.matches("[A-Za-z0-9_.-]+")||!names.add(v.name))throw new IllegalArgumentException("Unsafe or duplicate view name: "+v.name);long start=System.nanoTime();BufferedImage image=new Tracer(scene,v).render();ImageIO.write(image,"PNG",out.resolve(v.name+".png").toFile());double seconds=(System.nanoTime()-start)/1e9;String row=String.format(Locale.ROOT,"%s: %dx%d, %.3f s",v.name,v.width,v.height,seconds);System.out.println(row);record.add(row);index.add("- ["+v.name+"]("+v.name+".png) - "+line);}
        record.add(String.format(Locale.ROOT,"render total: %.3f s",(System.nanoTime()-total)/1e9));record.add("Plain-cube fallbacks: "+scene.assets.fallbacks);record.add("Missing textures: "+scene.assets.missing);record.add("Light sources: Blocks.<clinit> lightLevel lambdas and Properties.noOcclusion/noCollision/ofLegacyCopy, disassembled directly from the input jar by the JDK javap. Full model bounds determine full-cell occlusion for this preview's non-opaque-cell BFS.");record.add("Block metadata:");scene.assets.rules.used.forEach((k,v)->record.add(k+": "+v));Files.write(out.resolve("views.md"),index);Files.write(out.resolve("notes.txt"),record);}
    public static void main(String[]args)throws Exception{
        try { run(args); }
        catch(Exception | AssertionError ex){
            Path output=args.length==10?Path.of(args[9]):args.length==3&&args[0].equals("--fixture")?Path.of(args[2]):null;
            if(output!=null){try{Files.createDirectories(output);List<String>record=new ArrayList<>();record.add("arguments="+Arrays.toString(args));record.add("refusal="+ex);record.add("source sha256="+sha(Path.of("tools/preview/Preview.java")));for(String arg:args){Path input=Path.of(arg);if(Files.isRegularFile(input))record.add(input+" sha256="+sha(input));}Files.write(output.resolve("refusal-"+System.currentTimeMillis()+".txt"),record);}catch(Exception write){ex.addSuppressed(write);}}
            throw ex;
        }
    }
    static void run(String[]args)throws Exception{
        System.setProperty("java.awt.headless","true");ImageIO.setUseCache(false);System.setProperty("java.util.concurrent.ForkJoinPool.common.parallelism",Integer.toString(Math.min(8,Runtime.getRuntime().availableProcessors())));
        if(args.length>0&&args[0].equals("--fixture")){fixture(Path.of(args[1]),Path.of(args[2]));return;}
        if(args.length!=10){System.err.println("Usage: java Preview.java <commands> <x0> <y0> <z0> <x1> <y1> <z1> <client.jar> <views.txt> <output>\n       java Preview.java --fixture <client.jar> <output>");System.exit(2);}
        long start=System.nanoTime();Path commands=Path.of(args[0]),jar=Path.of(args[7]),views=Path.of(args[8]),out=Path.of(args[9]);World w=new World(Integer.parseInt(args[1]),Integer.parseInt(args[2]),Integer.parseInt(args[3]),Integer.parseInt(args[4]),Integer.parseInt(args[5]),Integer.parseInt(args[6]));w.replay(commands);
        try(Assets a=new Assets(jar)){Scene scene=new Scene(w,a);double prepare=(System.nanoTime()-start)/1e9;renderAll(scene,views,out);List<String>record=new ArrayList<>(List.of("commands="+commands.toAbsolutePath(),"commands sha256="+sha(commands),"box="+String.join(" ",Arrays.copyOfRange(args,1,7)),"jar="+jar.toAbsolutePath(),"jar sha256="+sha(jar),"views="+views.toAbsolutePath(),"views sha256="+sha(views),"source sha256="+sha(Path.of("tools/preview/Preview.java")),"arguments="+Arrays.toString(args),"java="+System.getProperty("java.version"),"prepare seconds="+prepare,"total seconds="+(System.nanoTime()-start)/1e9,"invariants: light range=0..15; refusals=none"));Files.write(out.resolve("run.txt"),record);System.out.printf(Locale.ROOT,"TOTAL %.3f s (prepare %.3f s)%n",(System.nanoTime()-start)/1e9,prepare);}
    }
    static void check(boolean pass,String message){if(!pass)throw new AssertionError(message);}
    static void fixture(Path jar,Path out)throws Exception{
        Files.createDirectories(out);Path commands=out.resolve("hut.txt"),views=out.resolve("views.txt");
        Files.writeString(commands,"""
            # 5 by 5 hut, with two-block spruce door and bottom-slab roof.
            fill 0 -60 0 4 -57 4 minecraft:stone_bricks
            fill 1 -60 1 3 -57 3 minecraft:air
            setblock 2 -60 0 minecraft:spruce_door[facing=north,half=lower,open=false,hinge=left]
            setblock 2 -59 0 minecraft:spruce_door[facing=north,half=upper,open=false,hinge=left]
            fill 0 -56 0 4 -56 4 minecraft:stone_brick_slab[type=bottom]
            setblock 2 -58 2 minecraft:lantern[hanging=true]
            setblock 5 -60 1 minecraft:stone_brick_stairs[facing=east,half=bottom,shape=straight]
            """);
        Files.writeString(views,"""
            iso NE day
            iso SW night
            plan top
            plan -58
            elev N
            elev E
            elev S
            elev W
            section x 2
            section z 2
            eye 2 -60 -3 0 0 70 day approach
            eye 2 -60 1 0 -15 70 night interior
            """);
        long start=System.nanoTime();World w=new World(-2,-61,-2,6,-55,6);w.replay(commands);
        try(Assets a=new Assets(jar)){Scene scene=new Scene(w,a);
                        check((a.water&255)>(a.water>>16&255),"plains water color must read as blue");check(a.shape(Block.parse("water"),0).alpha<1,"water is translucent");String[]emitters={"lantern","torch","soul_lantern","sea_lantern","glowstone","shroomlight","campfire[lit=true]","redstone_lamp[lit=true]","white_candle[candles=4,lit=true]","white_candle[candles=4,lit=false]","glow_lichen[north=true]","cave_vines[berries=true]","respawn_anchor[charges=4]","vault[vault_state=active]","trial_spawner[trial_spawner_state=active]","copper_bulb[lit=true]","exposed_copper_bulb[lit=true]","weathered_copper_bulb[lit=true]","oxidized_copper_bulb[lit=true]"};int[]levels={15,14,10,15,15,15,15,15,12,0,7,14,15,12,8,15,12,8,4};for(int i=0;i<emitters.length;i++){Block eb=Block.parse(emitters[i]);check(a.shape(eb,0).emission==levels[i],"jar-derived emission "+emitters[i]);}
            check(scene.light.get(2,-58,2)==15,"lantern cell must be 15");int[][]neighbors={{1,-58,2},{3,-58,2},{2,-57,2},{2,-59,2},{2,-58,1},{2,-58,3}};for(int[]q:neighbors)check(scene.light.get(q[0],q[1],q[2])==14,"lantern neighbor must be 14: "+Arrays.toString(q));
            check(scene.light.get(0,-58,2)==0,"opaque wall must stop light");
            renderAll(scene,views,out);for(String line:lines(views)){View v=parseView(line,w);BufferedImage im=ImageIO.read(out.resolve(v.name+".png").toFile());int ew=v.name.startsWith("eye")?1280:v.name.startsWith("iso")?134:72;int eh=v.name.startsWith("eye")?720:v.name.startsWith("iso")?137:v.name.startsWith("plan")?72:56;check(im.getWidth()==ew&&im.getHeight()==eh,"image size "+v.name+": "+im.getWidth()+"x"+im.getHeight()+" expected "+ew+"x"+eh);}
            BufferedImage before=ImageIO.read(out.resolve("iso-NE-day.png").toFile());w.fill(5,-60,1,5,-60,1,Block.parse("stone_brick_stairs[facing=west,half=bottom,shape=straight]"));Scene mutant=new Scene(w,a);BufferedImage after=new Tracer(mutant,parseView("iso NE day",w)).render();int changed=0;for(int y=0;y<before.getHeight();y++)for(int x=0;x<before.getWidth();x++)if(before.getRGB(x,y)!=after.getRGB(x,y))changed++;check(changed>0,"wrong stair facing must change image");ImageIO.write(after,"PNG",out.resolve("mutant-stairs-west.png").toFile());
            Path mutantCommands=out.resolve("hut-mutant.txt");Files.writeString(mutantCommands,Files.readString(commands).replace("facing=east,half=bottom,shape=straight","facing=west,half=bottom,shape=straight"));
            // Parser controls: slash/comments, order, clipping, state retention, and clear refusals.
            World p=new World(0,-61,0,1,-59,1);Path parser=out.resolve("parser.txt");Files.writeString(parser,"# comment\n/fill -3 -60 -3 3 -60 3 stone\nsetblock 0 -60 0 air\n");p.replay(parser);check(p.get(0,-60,0).equals(AIR)&&p.get(1,-60,1).id.equals("stone"),"replay order and clipping");check(p.get(8,-61,8).equals(GRASS)&&p.get(8,-62,8).equals(DIRT)&&p.get(8,-60,8).equals(AIR),"outside superflat");
            Files.write(out.resolve("fixture-result.txt"),List.of("PASS","lantern=15; six neighbors=14; opaque wall=0; worst light margin=0","all 12 image sizes asserted; 19 emitter/state controls asserted","wrong-facing image pixels changed="+changed,"commands sha256="+sha(commands),"mutant sha256="+sha(mutantCommands),"jar sha256="+sha(jar),"source sha256="+sha(Path.of("tools/preview/Preview.java")),"arguments=--fixture "+jar+" "+out,"refusals=none","seconds="+(System.nanoTime()-start)/1e9));System.out.println("FIXTURE PASS; wrong-facing pixels changed="+changed);
        }
    }
}
