export interface ReceiptArtifact {
  name: string;
  hash: string;
}

export interface WsReceipt {
  kind: "ws/receipt";
  tenant: string;
  workspace: string;
  trigger: string;       // deve casar com o atom_hash do request
  exit_code: number;
  duration_ms: number;
  stdout_hash?: string;
  stderr_hash?: string;
  artifacts?: ReceiptArtifact[];
  toolchain?: Record<string, unknown>;
}


