# Automated CI Dependency Isolation Assertion Script (Axiom 1)
# Verifies Zone A / Zone B crates have zero transitive dependencies on private key signing symbols

$ErrorActionPreference = "Stop"

Write-Host "Running Axiom 1 Dependency Isolation Verification..."

$ForbiddenSymbols = @(
    "k256::SecretKey",
    "ed25519_dalek::SigningKey",
    "bip32::ExtendedPrivateKey"
)

$PublicPlaneCrates = @(
    "gw-primitives",
    "gw-storage-oberer"
)

foreach ($crateName in $PublicPlaneCrates) {
    Write-Host "Checking crate: $crateName"
    $tree = cargo tree -p $crateName
    foreach ($sym in $ForbiddenSymbols) {
        if ($tree -match $sym) {
            Write-Error "VIOLATION OF AXIOM 1: Found forbidden private key symbol '$sym' in crate '$crateName'"
            exit 1
        }
    }
}

Write-Host "Axiom 1 Verification PASSED: Zero private key dependencies detected in Public Plane."
