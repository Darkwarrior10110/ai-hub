export interface SecureCredentialStore {
  saveCredential(providerId: string, credentialData: string): Promise<void>;
  getCredential(providerId: string): Promise<string | null>;
  deleteCredential(providerId: string): Promise<void>;
}
