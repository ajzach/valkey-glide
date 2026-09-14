/** Copyright Valkey GLIDE Project Contributors - SPDX Identifier: Apache-2.0 */
package glide.api.models.configuration;

/**
 * Supplies the current custom TLS trust chain for a GLIDE client.
 *
 * <p>The returned value must be a non-empty PEM bundle containing every trust anchor required at
 * that moment. During a planned CA rollover the bundle can contain both the old and new CA. GLIDE
 * validates the bundle before adopting it and retains the last-known-good trust chain if a later
 * invocation fails.
 *
 * <p><b>Thread safety:</b> implementations must be thread-safe. The callback runs from a native
 * blocking worker rather than the application command thread.
 *
 * <p><b>Promptness:</b> this callback is polled at the configured interval. It should use bounded
 * I/O and return promptly; a slow implementation delays trust-chain adoption.
 */
@FunctionalInterface
public interface RootCertificatesProvider {

    /**
     * Returns the complete PEM-encoded trust chain currently trusted by this client.
     *
     * @return a non-empty PEM certificate bundle
     * @throws Exception if the current trust chain cannot be retrieved
     */
    byte[] getRootCertificates() throws Exception;
}
