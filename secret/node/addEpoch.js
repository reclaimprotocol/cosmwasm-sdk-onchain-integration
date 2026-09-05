import { SecretNetworkClient, Wallet } from "secretjs";
import dotenv from "dotenv";
dotenv.config();

const wallet = new Wallet(process.env.MNEMONIC);

const secretjs = new SecretNetworkClient({
    chainId: "pulsar-3",
    url: "https://api.pulsar.scrttestnet.com",
    wallet: wallet,
    walletAddress: wallet.address,
});

let add_epoch = async () => {
    const contractAddress = process.env.CONTRACT_ADDRESS;
    const contractCodeHash = process.env.CONTRACT_CODE_HASH;
    const owner = "0x65954224b2ef6ec0546cbf2f716e8bba7ab5e22d"

    let tx = await secretjs.tx.compute.executeContract(
        {
            sender: wallet.address,
            contract_address: contractAddress,
            msg: {
                add_epoch: {
                    witness: [{ address: owner, host: "" }],
                    minimum_witness: "1",
                }
            },
            code_hash: contractCodeHash,
        },
        { gasLimit: 100_000 }
    );
    console.log(tx)
};

add_epoch();