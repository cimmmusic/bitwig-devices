import java.lang.reflect.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.io.*;

/**
 * Bridge to Bitwig's Nitro compiler for AST<->code conversion.
 * All access via reflection to survive obfuscation changes.
 *
 * Commands:
 *   binary-to-text <binary-file>     — convert binary AST to canonical text
 *   text-to-binary <text-file>       — compile text to binary AST
 *   validate <text-or-file>          — parse-check text (no compilation)
 */
public class NitroBridge {
    private static Class<?> nfClass;
    private static Method textParse;    // static NitroFile ?(byte[], String)
    private static Method binParse;     // static NitroFile ?(byte[])
    private static Method serialize;    // NitroFile.?() -> byte[]
    private static Method textPrint;    // NitroFile.?(txW)
    private static Constructor<?> txwCtor; // txW(Writer)

    static {
        try {
            nfClass = Class.forName("com.bitwig.nitro.NitroFile");

            // Find text parse: static NitroFile ?(byte[], String)
            for (Method m : nfClass.getDeclaredMethods()) {
                if (!Modifier.isStatic(m.getModifiers())) continue;
                if (!m.getReturnType().equals(nfClass)) continue;
                Class<?>[] p = m.getParameterTypes();
                if (p.length == 2 && p[0] == byte[].class && p[1] == String.class) {
                    textParse = m; m.setAccessible(true); break;
                }
            }

            // Find binary parse: static NitroFile ?(byte[])
            for (Method m : nfClass.getDeclaredMethods()) {
                if (!Modifier.isStatic(m.getModifiers())) continue;
                if (!m.getReturnType().equals(nfClass)) continue;
                Class<?>[] p = m.getParameterTypes();
                if (p.length == 1 && p[0] == byte[].class) {
                    binParse = m; m.setAccessible(true); break;
                }
            }

            // Find serialize: NitroFile.?() -> byte[]
            for (Method m : nfClass.getDeclaredMethods()) {
                if (Modifier.isStatic(m.getModifiers())) continue;
                if (m.getParameterCount() == 0 && m.getReturnType() == byte[].class) {
                    serialize = m; m.setAccessible(true); break;
                }
            }

            // Find text printer: NitroFile.?(txW) where txW has a Writer constructor
            for (Method m : nfClass.getDeclaredMethods()) {
                if (Modifier.isStatic(m.getModifiers())) continue;
                Class<?>[] p = m.getParameterTypes();
                if (p.length == 1 && p[0].getName().contains("txW")) {
                    m.setAccessible(true);
                    textPrint = m;
                    // Find txW(Writer) constructor
                    for (Constructor<?> c : p[0].getDeclaredConstructors()) {
                        c.setAccessible(true);
                        if (c.getParameterCount() == 1 && Writer.class.isAssignableFrom(c.getParameterTypes()[0])) {
                            txwCtor = c; break;
                        }
                    }
                    break;
                }
            }
        } catch (Exception e) {
            System.err.println("INIT_ERROR: " + e.getMessage());
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length < 2) {
            System.err.println("Usage: NitroBridge <command> <file-or-text>");
            System.err.println("Commands: binary-to-text, text-to-binary, validate");
            System.exit(2);
        }

        String cmd = args[0];
        String input = args[1];

        switch (cmd) {
            case "binary-to-text": {
                byte[] binary = Files.readAllBytes(Paths.get(input));
                Object file = binParse.invoke(null, binary);
                StringWriter sw = new StringWriter();
                Object writer = txwCtor.newInstance(sw);
                textPrint.invoke(file, writer);
                System.out.print(sw.toString());
                break;
            }
            case "text-to-binary": {
                byte[] textBytes;
                if (new File(input).exists()) {
                    textBytes = Files.readAllBytes(Paths.get(input));
                } else {
                    textBytes = input.getBytes(StandardCharsets.UTF_8);
                }
                Object file = textParse.invoke(null, textBytes, "(input)");
                byte[] binary = (byte[]) serialize.invoke(file);
                // Write to stdout or file
                if (args.length > 2) {
                    Files.write(Paths.get(args[2]), binary);
                    System.err.println("OK: " + binary.length + " bytes");
                } else {
                    System.out.write(binary);
                }
                break;
            }
            case "validate": {
                byte[] textBytes;
                if (new File(input).exists()) {
                    textBytes = Files.readAllBytes(Paths.get(input));
                } else {
                    textBytes = input.getBytes(StandardCharsets.UTF_8);
                }
                try {
                    textParse.invoke(null, textBytes, "(input)");
                    System.out.println("OK");
                } catch (InvocationTargetException e) {
                    System.err.println("FAIL: " + e.getCause().getMessage());
                    System.exit(1);
                }
                break;
            }
            default:
                System.err.println("Unknown command: " + cmd);
                System.exit(2);
        }
    }
}
