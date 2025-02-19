use super::g2::{encode_g2_point, extract_g2_input};
use crate::bls12_381_const::{
    G2_ADD_ADDRESS, G2_ADD_BASE_GAS_FEE, G2_ADD_INPUT_LENGTH, G2_INPUT_ITEM_LENGTH,
};
use crate::{u64_to_address, PrecompileWithAddress};
use crate::{PrecompileError, PrecompileOutput, PrecompileResult};
use blst::{
    blst_p2, blst_p2_add_or_double_affine, blst_p2_affine, blst_p2_from_affine, blst_p2_to_affine,
};
use primitives::Bytes;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(G2_ADD_ADDRESS), g2_add);

/// G2 addition call expects `512` bytes as an input that is interpreted as byte
/// concatenation of two G2 points (`256` bytes each).
///
/// Output is an encoding of addition operation result - single G2 point (`256`
/// bytes).
/// See also <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-addition>
pub(super) fn g2_add(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if G2_ADD_BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    if input.len() != G2_ADD_INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G2ADD input should be {G2_ADD_INPUT_LENGTH} bytes, was {}",
            input.len()
        ))
        .into());
    }

    // NB: There is no subgroup check for the G2 addition precompile.
    //
    // So we set the subgroup checks here to `false`
    let a_aff = &extract_g2_input(&input[..G2_INPUT_ITEM_LENGTH], false)?;
    let b_aff = &extract_g2_input(&input[G2_INPUT_ITEM_LENGTH..], false)?;

    let mut b = blst_p2::default();
    // SAFETY: `b` and `b_aff` are blst values.
    unsafe { blst_p2_from_affine(&mut b, b_aff) };

    let mut p = blst_p2::default();
    // SAFETY: `p`, `b` and `a_aff` are blst values.
    unsafe { blst_p2_add_or_double_affine(&mut p, &b, a_aff) };

    let mut p_aff = blst_p2_affine::default();
    // SAFETY: `p_aff` and `p` are blst values.
    unsafe { blst_p2_to_affine(&mut p_aff, &p) };

    let out = encode_g2_point(&p_aff);
    Ok(PrecompileOutput::new(G2_ADD_BASE_GAS_FEE, out))
}
