import { syncBindings } from './Bindings';

export enum Network {
  DOGECOIN = 1,
  LITECOIN = 2,
  BITCOIN = 3,
}

export function decodeBlock(tx: Uint8Array, network: Network): Bork[] {
  let borks = [];
  syncBindings.decode_block(tx, network, borks);
  return borks;
}

export class Bork {
  static get MAGIC(): Uint8Array {
    return syncBindings.magic_num();
  }

  encode(): Uint8Array[] {
    let parts = [];
    syncBindings.encode(this, parts);
    return parts;
  }
}

export class StandardBork extends Bork {
  content: string;
  nonce: number;

  constructor(content: string, nonce: number) {
    super();
    this.content = content;
    this.nonce = nonce;
  }
}

export class Extension extends StandardBork {
  index: number;

  constructor(content: string, nonce: number, index: number) {
    super(content, nonce);
    this.index = index;
  }
}

export abstract class ReferBork extends StandardBork {
  referenceId: Uint8Array;
  nonce: number;

  constructor(content: string, nonce: number, refId: Uint8Array) {
    super(content, nonce);
    this.referenceId = refId;
  }
}

export abstract class Comment extends ReferBork {
  constructor(content: string, nonce: number, refId: Uint8Array) {
    super(content, nonce, refId);
  }
}
export abstract class Rebork extends ReferBork {
  constructor(content: string, nonce: number, refId: Uint8Array) {
    super(content, nonce, refId);
  }
}