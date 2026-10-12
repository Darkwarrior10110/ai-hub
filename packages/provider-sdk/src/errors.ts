export class ProviderError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ProviderError';
  }
}
export class ProviderNotFound extends ProviderError { name = 'ProviderNotFound'; }
export class ProviderUnavailable extends ProviderError { name = 'ProviderUnavailable'; }
export class AuthenticationRequired extends ProviderError { name = 'AuthenticationRequired'; }
export class AuthenticationFailed extends ProviderError { name = 'AuthenticationFailed'; }
export class AuthenticationNotSupported extends ProviderError { name = 'AuthenticationNotSupported'; }
export class ConnectionFailed extends ProviderError { name = 'ConnectionFailed'; }
export class RateLimited extends ProviderError { name = 'RateLimited'; }
export class CapabilityNotSupported extends ProviderError { name = 'CapabilityNotSupported'; }
export class ProviderConfigurationError extends ProviderError { name = 'ProviderConfigurationError'; }
