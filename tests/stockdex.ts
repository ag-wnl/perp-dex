import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Stockdex } from "../target/types/stockdex";
import { assert } from "chai";

describe("stockdex", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Stockdex as Program<Stockdex>;
  const admin = provider.wallet;

  it("Is initialized!", async () => {
    const params = {
      minSignatures: 1,
      allowSwap: true,
      allowAddLiquidity: true,
      allowRemoveLiquidity: true,
      allowOpenPosition: true,
      allowClosePosition: true,
      allowPnlWithdrawal: true,
      allowCollateralWithdrawal: true,
      allowSizeChange: true,
    };

    const [multisig] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("multisig")],
      program.programId
    );

    const [transferAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("transfer_authority")],
      program.programId
    );

    const [perpetuals] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("perpetuals")],
      program.programId
    );

    const [programData] = anchor.web3.PublicKey.findProgramAddressSync(
      [program.programId.toBuffer()],
      new anchor.web3.PublicKey("BPFLoaderUpgradeab1e11111111111111111111111")
    );

    // Add your test here.
    const tx = await program.methods
      .init(params)
      .accounts({
        upgradeAuthority: admin.publicKey,
        multisig,
        transferAuthority,
        perpetuals,
        perpetualsProgramData: programData,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.Spl.tokenProgram.programId,
      })
      .remainingAccounts([
        { pubkey: admin.publicKey, isSigner: false, isWritable: false },
      ])
      .rpc();

    console.log("Your transaction signature", tx);

    const multisigAccount = await program.account.multisig.fetch(multisig);
    assert.equal(multisigAccount.minSignatures, params.minSignatures);
    assert.equal(
      multisigAccount.signers[0].toString(),
      admin.publicKey.toString()
    );

    const perpetualsAccount = await program.account.perpetuals.fetch(
      perpetuals
    );
    assert.isTrue(perpetualsAccount.permissions.allowAddLiquidity);
  });
});
