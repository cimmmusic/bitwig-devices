import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;

/**
 * Runs the pinned Bitwig 6.1.3 Nitro front end, including semantic analysis,
 * on plain-text sources without launching the application.
 *
 * Usage: java -DinstallationRoot=<Bitwig dir> -cp <probe>:<bitwig.jar>:<libs.jar>
 *        [-Dnitro.llvm.output=<temporary directory>] NitroSemanticProbe <source.nitro>...
 *
 * Every source is analysed on its own, one after another. Bitwig analyses the
 * cores of a device in parallel, and a cold parallel analysis can fail with
 * "Recursion in semantic analysis"; a serial run reports only source defects.
 * Output: one line per source, "OK<TAB>path" or "ERROR<TAB>path<TAB>message".
 * Host context such as the connected channel count is not modelled, so a
 * failing static_assert is context-dependent rather than a source defect.
 * The obfuscated entry points below are bound to the pinned JAR hash.
 */
public class NitroSemanticProbe {
    // Obfuscated names differ per build and can collide (macOS 6.1.4 also has
    // an unrelated azQ). -Dnitro.names=macos-6.1.4 selects the names from
    // Patchbay's windows-6.1.3__macos-6.1.4 remap; the default is Windows 6.1.3.
    static final boolean MACOS_614 = "macos-6.1.4".equals(System.getProperty("nitro.names"));

    static Class<?> cls(String windows613, String macos614) throws ClassNotFoundException {
        return Class.forName(MACOS_614 ? macos614 : windows613);
    }

    public static void main(String[] args) throws Exception {
        Class<?> flags = cls("com.bitwig.nitro.Xf", "com.bitwig.nitro.vna");
        Object noFlags = flags.getConstructors()[0].newInstance(
            false, false, false, false, false, false, false, false, false, false, false, false);
        // Compiler settings: revision, chunk size, block size, sample rate,
        // oversampling factor, FFT order, ..., zero-init, absolute paths, flags.
        // A 1024-frame chunk turned audio ports into f32<1024> and rejected the
        // host-accepted Growl and Normalizer sources; 32 accepts them.
        Object settings = cls("com.bitwig.nitro.EWg", "com.bitwig.nitro.CHR").getConstructors()[0]
            .newInstance(0, 32, 1, 44100, 1.0, 0, 0, false, false, noFlags);

        // Plain-text code source of a Nitro atom. Its outer atom is only used
        // for identity, so an unconstructed instance is sufficient.
        Field unsafeField = Class.forName("sun.misc.Unsafe").getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        Object unsafe = unsafeField.get(null);
        Object atom = unsafe.getClass().getMethod("allocateInstance", Class.class)
            .invoke(unsafe, cls("azQ", "BgP"));
        Constructor<?> textSource = cls("BYC", "FRG").getDeclaredConstructors()[0];
        textSource.setAccessible(true);

        Class<?> codeSource = cls("com.bitwig.nitro.TJZ", "com.bitwig.nitro.IPO");
        Method compile = null;
        for (Method method : cls("com.bitwig.nitro.PI", "com.bitwig.nitro.FDA").getDeclaredMethods()) {
            Class<?>[] types = method.getParameterTypes();
            if (Modifier.isStatic(method.getModifiers()) && types.length == 8
                    && types[0] == String.class && types[1] == codeSource) {
                compile = method;
            }
        }
        if (compile == null) {
            throw new IllegalStateException("Nitro compile entry point not found in this JAR");
        }
        compile.setAccessible(true);

        String llvmOutput = System.getProperty("nitro.llvm.output");
        Field llvmText = null;
        if (llvmOutput != null) {
            // Exact Windows 6.1.3 compiler result field holding emitted LLVM text.
            String llvmField = MACOS_614 ? "Hnt" : "HEY";
            llvmText = cls("com.bitwig.nitro.O2W", "com.bitwig.nitro.jiw").getDeclaredField(llvmField);
            llvmText.setAccessible(true);
        }
        for (int index = 0; index < args.length; index++) {
            String path = Path.of(args[index]).toAbsolutePath().toString().replace('\\', '/');
            try {
                Object source = textSource.newInstance(atom, Files.readAllBytes(Path.of(path)), path);
                Object compiled = compile.invoke(null, path, source, settings,
                    new ArrayList<>(), new ArrayList<>(), new ArrayList<>(), new ArrayList<>(), path);
                if (llvmText != null) {
                    String ir = (String) llvmText.get(compiled);
                    if (ir == null || ir.isBlank()) {
                        throw new IllegalStateException("Nitro returned no LLVM IR for " + path);
                    }
                    Files.writeString(Path.of(llvmOutput, index + ".ll"), ir);
                }
                System.out.println("OK\t" + path);
            } catch (InvocationTargetException failure) {
                Throwable cause = failure.getCause();
                // The compiler wraps front-end errors; report the innermost message.
                while (cause.getCause() != null && cause.getCause() != cause) {
                    cause = cause.getCause();
                }
                String message = String.valueOf(cause.getMessage()).replace('\n', ' ').replace('\t', ' ');
                System.out.println("ERROR\t" + path + "\t" + cause.getClass().getSimpleName() + ": " + message);
            }
        }
    }
}
