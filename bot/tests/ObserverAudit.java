package pathtracing;
import java.nio.file.Path;
import java.util.List;
import java.util.jar.JarFile;
import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;

/** Verify passive overlay sites against the exact frozen flying binary. */
public final class ObserverAudit {
    private static Object value(AnnotationNode a,String key) {
        for(int i=0;i<a.values.size();i+=2) if(a.values.get(i).equals(key)) return a.values.get(i+1);
        throw new AssertionError("Missing annotation "+key);
    }
    private static ClassNode read(JarFile jar,String name) throws Exception {
        var entry=jar.getJarEntry(name+".class"); if(entry==null) throw new AssertionError("Missing class "+name);
        var out=new ClassNode(); try(var in=jar.getInputStream(entry)) { new ClassReader(in).accept(out,0); } return out;
    }
    public static void main(String[] args) throws Exception {
        int count=0;
        try(var overlay=new JarFile(args[0]);var baseline=new JarFile(args[1])) {
            for(String name:List.of("HopBaselinePointerMixin","HopBaselineTrackMixin","HopBaselineBotMixin","HopBaselineRestoreMixin")) {
                var mixin=read(overlay,"pathtracing/observer/"+name);
                var declaration=mixin.invisibleAnnotations.stream().filter(a->a.desc.endsWith("/Mixin;")).findFirst().orElseThrow();
                var target=read(baseline,((String)((List<?>)value(declaration,"targets")).getFirst()).replace('.','/'));
                for(var handler:mixin.methods) {
                    if(handler.visibleAnnotations==null) continue;
                    for(var annotation:handler.visibleAnnotations) {
                        if(!annotation.desc.endsWith("/Inject;") && !annotation.desc.endsWith("/Redirect;")) continue;
                        String selector=(String)((List<?>)value(annotation,"method")).getFirst();
                        var matches=target.methods.stream().filter(m->m.name.equals(selector)).toList();
                        if(matches.size()!=1) throw new AssertionError("Ambiguous/missing "+name+":"+selector);
                        var method=matches.getFirst();
                        if((handler.access&Opcodes.ACC_STATIC)!=(method.access&Opcodes.ACC_STATIC)) throw new AssertionError("Static mismatch "+name);
                        if(annotation.desc.endsWith("/Redirect;")) {
                            Object rawAt=value(annotation,"at");
                            AnnotationNode at=(AnnotationNode)(rawAt instanceof List<?> list ? list.getFirst() : rawAt);
                            String wanted=(String)value(at,"target"); boolean found=false;
                            for(var insn:method.instructions) if(insn instanceof MethodInsnNode call && wanted.equals("L"+call.owner+";"+call.name+call.desc)) found=true;
                            if(!found) throw new AssertionError("Missing original call site "+wanted);
                        }
                        count++;
                    }
                }
            }
        }
        if(count!=4) throw new AssertionError("Expected four passive original-binary sites, got "+count);
        System.out.println("PASS: four ObserverOnly sites match the frozen original binary");
    }
}
