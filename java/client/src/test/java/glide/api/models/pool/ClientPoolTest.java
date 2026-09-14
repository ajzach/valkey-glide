/** Copyright Valkey GLIDE Project Contributors - SPDX-License-Identifier: Apache-2.0 */
package glide.api.models.pool;

import static org.junit.jupiter.api.Assertions.assertThrows;

import glide.api.models.configuration.AdvancedGlideClientConfiguration;
import glide.api.models.configuration.GlideClientConfiguration;
import glide.api.models.configuration.TlsAdvancedConfiguration;
import org.junit.jupiter.api.Test;

class ClientPoolTest {

    @Test
    void rejectsDynamicRootCertificatesProvider() {
        GlideClientConfiguration clientConfig =
                GlideClientConfiguration.builder()
                        .useTLS(true)
                        .advancedConfiguration(
                                AdvancedGlideClientConfiguration.builder()
                                        .tlsAdvancedConfiguration(
                                                TlsAdvancedConfiguration.builder()
                                                        .useRootCertificatesProvider(() -> new byte[] {1}, 60)
                                                        .build())
                                        .build())
                        .build();

        ClientPoolConfig poolConfig = ClientPoolConfig.builder().clientConfig(clientConfig).build();

        assertThrows(IllegalArgumentException.class, () -> ClientPool.create(poolConfig));
    }
}
