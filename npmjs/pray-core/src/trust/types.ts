export interface TrustPolicy {
  default: TrustRule;
  rules: TrustRule[];
}

export interface TrustRule {
  match_prefix?: string;
  allow?: boolean;
  require_signed_commit?: boolean;
  allowed_signing_keys?: string[];
  allowed_host_keys?: string[];
  allowed_publishers?: string[];
}
