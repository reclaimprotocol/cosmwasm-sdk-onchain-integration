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

let verify_proof = async () => {
    const contractAddress = process.env.CONTRACT_ADDRESS;
    const contractCodeHash = process.env.CONTRACT_CODE_HASH;

    const owner = "0xe70415eb011253b6721d4f9149dd525d6afe370f"

    const claimInfo = {
        "provider": "provider",
        "parameters": "param",
        "context": "{}",
    }

    const identifier = "0xa6db2030140d1a1297ea836cf1fb0a1b467c5c21499dc0cd08dba63d62a6fdcc"

    // const id = [53,148,134,250,217,11,186,55,221,14,162,179,148,70,207,252,19,30,22,135,213,37,64,50,8,167,159,10,37,141,217,151]
    // const signatures = [[118,28,143,27,79,77,36,104,89,153,205,10,106,67,128,12,189,95,188,181,207,184,61,179,116,203,27,45,119,19,206,216,1,204,78,246,206,48,128,188,174,29,179,235,220,63,91,54,150,196,193,218,197,82,183,235,30,67,72,218,125,107,173,34,1]]
    
    const complete_signature = {
        signature: "d8076039793e7014a9fd746b8531530d52d66c4c622e346ca1f157323348cd5e53cd95e00f88e211b6fddc3bee78c8fbbecc4469000620764f611e4d2b7dabde",
        recovery_param: 0,
    }

    const signedClaim = {
        "claim": {
            "identifier": identifier,
            "owner": owner,
            "epoch": "1",
            "timestamp_s": 1709553706
        },
        "signatures": [complete_signature],
    }

    
    let tx2 = await secretjs.tx.compute.executeContract(
        {
            sender: wallet.address,
            contract_address: contractAddress,
            msg: {
                verify_proof: {
                    claim_info: claimInfo,
                    signed_claim: signedClaim,
                }
            },
            code_hash: contractCodeHash,
        },
        { gasLimit: 100_000 }
    );

    console.log(tx2)
};

verify_proof();