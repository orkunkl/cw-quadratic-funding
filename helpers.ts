/*
 * This is a set of helpers meant for use with @cosmjs/cli
 * With these you can easily use the cw20 contract without worrying about forming messages and parsing queries.
 *
 * Usage: npx @cosmjs/cli --init https://raw.githubusercontent.com/CosmWasm/cosmwasm-examples/master/nameservice/helpers.ts
 *
 * Create a client:
 *   const client = await useOptions(hackatomOptions).setup(password);
 *   await client.getAccount()
 *
 * Get the mnemonic:
 *   await useOptions(hackatomOptions).recoverMnemonic(password)
 *
 * If you want to use this code inside an app, you will need several imports from https://github.com/CosmWasm/cosmjs
 */

const path = require("path");

interface Options {
  readonly httpUrl: string
  readonly networkId: string
  readonly feeToken: string
  readonly gasPrice: GasPrice
  readonly bech32prefix: string
  readonly hdPath: readonly Slip10RawIndex[]
  readonly faucetUrl?: string
  readonly defaultKeyFile: string
  readonly gasLimits: Partial<GasLimits<CosmWasmFeeTable>> // only set the ones you want to override
}

const hackatomOptions: Options = {
  httpUrl: 'https://lcd.heldernet.cosmwasm.com',
  networkId: 'hackatom-wasm',
  gasPrice:  GasPrice.fromString("0.025ucosm"),
  bech32prefix: 'cosmos',
  feeToken: 'ucosm',
  faucetUrl: 'https://faucet.heldernet.cosmwasm.com/credit',
  hdPath: makeCosmoshubPath(0),
  defaultKeyFile: path.join(process.env.HOME, ".heldernet.key"),
  gasLimits: {
    upload: 1500000,
    init: 600000,
    register:800000,
    transfer: 80000,
  },
}

interface Network {
  setup: (password: string, filename?: string) => Promise<SigningCosmWasmClient>
  recoverMnemonic: (password: string, filename?: string) => Promise<string>
}

const useOptions = (options: Options): Network => {

  const loadOrCreateWallet = async (options: Options, filename: string, password: string): Promise<Secp256k1HdWallet> => {
    let encrypted: string;
    try {
      encrypted = fs.readFileSync(filename, 'utf8');
    } catch (err) {
      // generate if no file exists
      const wallet = await Secp256k1HdWallet.generate(12, options.hdPath, options.bech32prefix);
      const encrypted = await wallet.serialize(password);
      fs.writeFileSync(filename, encrypted, 'utf8');
      return wallet;
    }
    // otherwise, decrypt the file (we cannot put deserialize inside try or it will over-write on a bad password)
    const wallet = await Secp256k1HdWallet.deserialize(encrypted, password);
    return wallet;
  };

  const connect = async (
    wallet: Secp256k1HdWallet,
    options: Options
  ): Promise<SigningCosmWasmClient> => {
    const [{ address }] = await wallet.getAccounts();

    const client = new SigningCosmWasmClient(
      options.httpUrl,
      address,
      wallet,
      hackatomOptions.gasPrice,
      hackatomOptions.gasLimits,
    );
    return client;
  };

  const hitFaucet = async (
    faucetUrl: string,
    address: string,
    denom: string
  ): Promise<void> => {
    await axios.post(faucetUrl, { denom, address });
  }

  const setup = async (password: string, filename?: string): Promise<SigningCosmWasmClient> => {
    const keyfile = filename || options.defaultKeyFile;
    const wallet = await loadOrCreateWallet(hackatomOptions, keyfile, password);
    const client = await connect(wallet, hackatomOptions);

    // ensure we have some tokens
    if (options.faucetUrl) {
      const account = await client.getAccount();
      if (!account) {
        console.log(`Getting ${options.feeToken} from faucet`);
        await hitFaucet(options.faucetUrl, client.senderAddress, options.feeToken);
      }
    }

    return client;
  }

  const recoverMnemonic = async (password: string, filename?: string): Promise<string> => {
    const keyfile = filename || options.defaultKeyFile;
    const wallet = await loadOrCreateWallet(hackatomOptions, keyfile, password);
    return wallet.mnemonic;
  }

  return {setup, recoverMnemonic};
}

interface Config {
  readonly admin: string,
  readonly create_proposal_whitelist?: string[],
  readonly vote_proposal_whitelist?: string[],
  readonly voting_period: Expiration,
  readonly proposal_period: Expiration,
  readonly budget: Coin,
  readonly algorithm: QFAlgorithm,
}


interface QFAlgorithm {
  readonly type?: string // TODO apply more strict enum check
}

interface InitMsg {
  readonly admin: string,
  readonly create_proposal_whitelist?: string[],
  readonly vote_proposal_whitelist?: string[],
  readonly voting_period: Expiration,
  readonly proposal_period: Expiration,
  readonly budget_denom: string,
  readonly algorithm: QFAlgorithm,
}

interface Proposal {
  readonly id: number,
  readonly title: string,
  readonly description: string,
  readonly metadata?: BinaryType,
  readonly fund_address: string,
}

interface Vote {
  readonly proposalId: number,
  readonly voter: string,
  readonly fund: Coin,
}

type Expiration = {readonly at_height: string} | {readonly at_time: string} | {readonly never: {}};

interface QuadraticFundingInstance {
  readonly contractAddress: string

  // queries
  proposal: (id: number) => Promise<Proposal>

  // actions
  createProposal: (name: string, title: string, description: string, fundAddress: string , amount: readonly Coin[], metadata?: BinaryType) => Promise<string>
  voteProposal: (proposalId: number, amount: readonly Coin[]) => Promise<any>
  triggerDistribution: () => Promise<any>
}

interface QuadraticFundingContract{
  upload: () => Promise<number>

  instantiate: (codeId: number, initMsg: Record<string, string>, label: string) => Promise<QuadraticFundingInstance>

  use: (contractAddress: string) => QuadraticFundingInstance
}

const QuadraticFunding = (client: SigningCosmWasmClient): QuadraticFundingContract=> {
  const use = (contractAddress: string): QuadraticFundingInstance => {
    const proposal = async (id: number): Promise<Proposal> => {
      return client.queryContractSmart(contractAddress, {proposal: { id}});
    };

      const createProposal = async (name: string, title: string, description: string , fundAddress: string , amount: readonly Coin[], metadata?: BinaryType): Promise<string> => {
      const result = await client.execute(contractAddress, {create_proposal: { name, title, description, fundAddress, metadata}}, "", amount);
      return result.transactionHash;
    };

    const voteProposal = async (proposalId: number, amount: readonly Coin[]): Promise<any> => {
      const result = await client.execute(contractAddress, {vote: { proposalId }}, "", amount);
      return result.transactionHash;
    };

    const triggerDistribution = async (): Promise<any> => {
      const result = await client.execute(contractAddress, {trigger_distribution: {}}, "");
      return result.transactionHash;
    };

    return {
      contractAddress,
      proposal,
      createProposal,
      voteProposal,
      triggerDistribution
    };
  }

  const downloadWasm = async (url: string): Promise<Uint8Array> => {
    const r = await axios.get(url, { responseType: 'arraybuffer' })
    if (r.status !== 200) {
      throw new Error(`Download error: ${r.status}`)
    }
    return r.data
  }

  const upload = async (): Promise<number> => {
    const meta = {
      source: "https://github.com/CosmWasm/cosmwasm-examples/tree/nameservice-0.7.0/nameservice",
      builder: "cosmwasm/rust-optimizer:0.10.4"
    };
    const sourceUrl = "https://github.com/CosmWasm/cosmwasm-examples/releases/download/nameservice-0.7.0/contract.wasm";
    const wasm = await downloadWasm(sourceUrl);
    const result = await client.upload(wasm, meta);
    return result.codeId;
  }

  const instantiate = async (codeId: number, initMsg: Record<string, string>,label: string): Promise<QuadraticFundingInstance> => {
    const result = await client.instantiate(codeId, initMsg, label, { memo: `Init ${label}`});
    return use(result.contractAddress);
  }

  return { upload, instantiate, use };
}

// Demo:
// const client = await useOptions(hackatomOptions).setup(PASSWORD);
// const { address } = await client.getAccount()
// const factory = QuadraticFunding(client)
//
// const codeId = await factory.upload();
// codeId -> 12
// const initMsg = { admin: "cosmos1z5huy8upud7jm4p62h3m2dtuae7anfnwx3mx8w", voting_period: { at_height: "257600" }, proposal_period: { at_height: 257600 }, budget_denom: "ucosm", quadratic_funding_algorithm: {capital_constrained_liberal_radicalism: {params: "param"}}}
// const contract = await factory.instantiate(90, initMsg, "cw1-subkey test")
// contract.contractAddress -> 'coral1267wq2zk22kt5juypdczw3k4wxhc4z47mug9fd'
//
// OR
//
// const contract = factory.use('coral1267wq2zk22kt5juypdczw3k4wxhc4z47mug9fd')
//
// const randomAddress = 'cosmos12my0yfs9ft4kafrzy0p2r7dn2ppd8zu65ll0ay'
//
// contract.config()
// contract.register("name", "some metadata", [{"denom": "ucosm", amount: "4000" }])
// contract.record("name")
// contract.transfer("name", randomAddress, [{"denom": "ushell", amount: "2000" }])
//
