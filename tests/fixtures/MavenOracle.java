import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import org.apache.maven.artifact.versioning.ComparableVersion;

/** Compare tab-separated version pairs with the supplied Apache Maven artifact. */
public final class MavenOracle {
    public static void main(String[] args) throws Exception {
        try (BufferedReader input = new BufferedReader(
                new InputStreamReader(System.in, StandardCharsets.UTF_8))) {
            String line;
            while ((line = input.readLine()) != null) {
                String[] pair = line.split("\t", -1);
                if (pair.length != 2) {
                    throw new IllegalArgumentException("Expected two tab-separated versions");
                }
                ComparableVersion left = new ComparableVersion(pair[0]);
                ComparableVersion right = new ComparableVersion(pair[1]);
                System.out.println(Integer.signum(left.compareTo(right)));
            }
        }
    }
}
