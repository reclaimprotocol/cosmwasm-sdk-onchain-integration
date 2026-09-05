"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs = __importStar(require("fs"));
const dotenv_1 = __importDefault(require("dotenv"));
dotenv_1.default.config();
const nibijs_1 = require("@nibiruchain/nibijs");
const contract_wasm = fs.readFileSync("../../reclaim_cosmwasm.wasm");
const func = () => __awaiter(void 0, void 0, void 0, function* () {
    const chain = (0, nibijs_1.Testnet)();
    let signer = yield (0, nibijs_1.newSignerFromMnemonic)(process.env.MNEMONIC || "");
    let accs = yield signer.getAccounts();
    let addr = accs[0].address;
    const txClient = yield nibijs_1.NibiruTxClient.connectWithSigner(chain.endptTm, signer);
    let storeResp = yield txClient.wasmClient.upload(addr, contract_wasm, "auto");
    let codeId = storeResp.codeId;
    let insResponse = yield txClient.wasmClient.instantiate(addr, codeId, { owner: addr }, "local0.1.0", "auto");
    let contractAddress = insResponse.contractAddress;
    console.log("Contract Address: ", contractAddress);
    console.log("Code Id: ", codeId);
    const reclaim = "0x244897572368eadf65bfbc5aec98d8e5443a9072";
    yield txClient.wasmClient.execute(addr, contractAddress, {
        add_epoch: {
            witness: [{ address: reclaim, host: "" }],
            minimum_witness: "1",
        },
    }, "auto");
    yield txClient.wasmClient.execute(addr, contractAddress, {
        add_epoch: {
            witness: [{ address: reclaim, host: "" }],
            minimum_witness: "1",
        },
    }, "auto");
    const owner = "0xe4c20c9f558160ec08106de300326f7e9c73fb7f";
    const claimInfo = {
        provider: "http",
        parameters: '{"body":"","geoLocation":"in","method":"GET","responseMatches":[{"type":"contains","value":"_steamid\\">Steam ID: 76561199632643233</div>"}],"responseRedactions":[{"jsonPath":"","regex":"_steamid\\">Steam ID: (.*)</div>","xPath":"id(\\"responsive_page_template_content\\")/div[@class=\\"page_header_ctn\\"]/div[@class=\\"page_content\\"]/div[@class=\\"youraccount_steamid\\"]"}],"url":"https://store.steampowered.com/account/"}',
        context: '{"contextAddress":"user\'s address","contextMessage":"for acmecorp.com on 1st january"}',
    };
    const identifier = "0x531322a6c34e5a71296a5ee07af13f0c27b5b1e50616f816374aff6064daaf55";
    const signedClaim = {
        claim: {
            identifier: identifier,
            owner: owner,
            epoch: 1,
            timestampS: 1710157447,
        },
        signatures: [
            "0x52e2a591f51351c1883559f8b6c6264b9cb5984d0b7ccc805078571242166b357994460a1bf8f9903c4130f67d358d7d6e9a52df9a38c51db6a10574b946884c1b",
        ],
    };
    const proof = {
        claimInfo: claimInfo,
        signedClaim: signedClaim,
    };
    yield txClient.wasmClient.execute(addr, contractAddress, {
        verify_proof: {
            proof: proof,
        },
    }, "auto");
});
func();
//# sourceMappingURL=index.js.map