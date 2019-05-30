import { syncBindings } from './Bindings';

export class Bork {
  static get MAGIC(): Uint8Array {
    return syncBindings.magic_num();
  }

  encode(): Uint8Array {
    return syncBindings.encode(this);
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