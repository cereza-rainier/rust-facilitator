"""
Python bindings for x402 Rust Facilitator
Uses ctypes to call the Rust library via FFI

This demonstrates how ANY language can use our Rust library!
"""

import ctypes
import json
import platform
from pathlib import Path
from typing import Optional, Dict, Any

class VerifyResult:
    """Result of payment verification"""
    def __init__(self, is_valid: bool, error_message: Optional[str] = None, payer: Optional[str] = None):
        self.is_valid = is_valid
        self.error_message = error_message
        self.payer = payer
    
    def __repr__(self):
        if self.is_valid:
            return f"VerifyResult(valid=True, payer='{self.payer}')"
        else:
            return f"VerifyResult(valid=False, error='{self.error_message}')"


class CVerifyResult(ctypes.Structure):
    """C-compatible result structure"""
    _fields_ = [
        ("is_valid", ctypes.c_bool),
        ("error_message", ctypes.c_char_p),
        ("payer", ctypes.c_char_p),
    ]


class X402Facilitator:
    """
    Python wrapper for x402 Rust Facilitator
    
    Example:
        facilitator = X402Facilitator()
        
        payment = {
            "x402_version": 1,
            "scheme": "exact",
            "network": "solana-devnet",
            "payload": {"transaction": "..."}
        }
        
        requirements = {
            "scheme": "exact",
            "network": "solana-devnet",
            # ... other fields
        }
        
        result = facilitator.verify(payment, requirements)
        if result.is_valid:
            print(f"Payment verified! Payer: {result.payer}")
        else:
            print(f"Verification failed: {result.error_message}")
    """
    
    def __init__(self, lib_path: Optional[str] = None):
        """
        Initialize the facilitator
        
        Args:
            lib_path: Path to the shared library. If None, auto-detects.
        """
        if lib_path is None:
            lib_path = self._find_library()
        
        # Load the shared library
        self.lib = ctypes.CDLL(lib_path)
        
        # Define function signatures
        self._setup_functions()
        
        # Initialize the library
        result = self.lib.x402_init()
        if result != 0:
            raise RuntimeError(f"Failed to initialize x402 library: {result}")
        
        print(f"✅ x402 Facilitator FFI loaded (version: {self.version()})")
    
    def _find_library(self) -> str:
        """Auto-detect the shared library path"""
        system = platform.system()
        
        # Library filename by platform
        if system == "Darwin":  # macOS
            lib_name = "libx402_facilitator.dylib"
        elif system == "Linux":
            lib_name = "libx402_facilitator.so"
        elif system == "Windows":
            lib_name = "x402_facilitator.dll"
        else:
            raise RuntimeError(f"Unsupported platform: {system}")
        
        # Search paths
        base_path = Path(__file__).parent.parent.parent.parent
        search_paths = [
            base_path / "target" / "release" / lib_name,
            base_path / "target" / "debug" / lib_name,
            Path(lib_name),  # Current directory
        ]
        
        for path in search_paths:
            if path.exists():
                print(f"📦 Found library at: {path}")
                return str(path)
        
        raise FileNotFoundError(
            f"Could not find {lib_name}. Build it first with:\n"
            f"  cargo build --release\n"
            f"Searched: {[str(p) for p in search_paths]}"
        )
    
    def _setup_functions(self):
        """Define C function signatures"""
        # x402_init
        self.lib.x402_init.argtypes = []
        self.lib.x402_init.restype = ctypes.c_int
        
        # x402_version
        self.lib.x402_version.argtypes = []
        self.lib.x402_version.restype = ctypes.c_char_p
        
        # x402_verify_payment
        self.lib.x402_verify_payment.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        self.lib.x402_verify_payment.restype = CVerifyResult
        
        # x402_free_string
        self.lib.x402_free_string.argtypes = [ctypes.c_char_p]
        self.lib.x402_free_string.restype = None
        
        # x402_free_result
        self.lib.x402_free_result.argtypes = [CVerifyResult]
        self.lib.x402_free_result.restype = None
    
    def version(self) -> str:
        """Get library version"""
        version_bytes = self.lib.x402_version()
        return version_bytes.decode('utf-8')
    
    def verify(self, payment: Dict[str, Any], requirements: Dict[str, Any]) -> VerifyResult:
        """
        Verify a payment
        
        Args:
            payment: Payment payload dict
            requirements: Payment requirements dict
        
        Returns:
            VerifyResult with is_valid, error_message, and payer
        """
        # Convert to JSON
        payment_json = json.dumps(payment).encode('utf-8')
        requirements_json = json.dumps(requirements).encode('utf-8')
        
        # Call C function
        c_result = self.lib.x402_verify_payment(payment_json, requirements_json)
        
        # Convert to Python result
        is_valid = c_result.is_valid
        
        error_message = None
        if c_result.error_message:
            error_message = c_result.error_message.decode('utf-8')
        
        payer = None
        if c_result.payer:
            payer = c_result.payer.decode('utf-8')
        
        # Create result
        result = VerifyResult(is_valid, error_message, payer)
        
        # Free C memory
        self.lib.x402_free_result(c_result)
        
        return result


# Example usage
if __name__ == "__main__":
    import time
    
    print("=" * 60)
    print("🦀 x402 Rust Facilitator - Python FFI Demo")
    print("=" * 60)
    print()
    
    # Initialize
    facilitator = X402Facilitator()
    print()
    
    # Example payment
    payment = {
        "x402_version": 1,
        "scheme": "exact",
        "network": "solana-devnet",
        "payload": {
            "transaction": "test_transaction_base64"
        },
        "timestamp": int(time.time())
    }
    
    requirements = {
        "scheme": "exact",
        "network": "solana-devnet",
        "max_amount_required": "1000000",
        "asset": "SOL",
        "pay_to": "recipient_address",
        "resource": "/api/premium",
        "description": "Premium API Access",
        "mime_type": "application/json",
        "max_timeout_seconds": 30,
        "extra": {
            "fee_payer": "fee_payer_address"
        }
    }
    
    print("📤 Verifying payment...")
    print(f"  Network: {payment['network']}")
    print(f"  Scheme: {payment['scheme']}")
    print()
    
    # Verify
    start = time.time()
    result = facilitator.verify(payment, requirements)
    duration = (time.time() - start) * 1000
    
    print(f"⏱️  Verification took: {duration:.2f}ms")
    print()
    
    if result.is_valid:
        print(f"✅ Payment VALID")
        print(f"   Payer: {result.payer}")
    else:
        print(f"❌ Payment INVALID")
        print(f"   Reason: {result.error_message}")
    
    print()
    print("=" * 60)
    print("🎯 Key Takeaway:")
    print("   You just called Rust code from Python via FFI!")
    print("   This works for Go, Java, Ruby, Node.js, etc.")
    print("=" * 60)

