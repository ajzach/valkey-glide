//go:build tools

package glide

// This file ensures rustbin platform directories are included when using
// `go mod vendor`. It is generated during releases.
import (
	_ "github.com/valkey-io/valkey-glide/go/v2/rustbin/aarch64-unknown-linux-gnu"
	_ "github.com/valkey-io/valkey-glide/go/v2/rustbin/x86_64-unknown-linux-gnu"
)
