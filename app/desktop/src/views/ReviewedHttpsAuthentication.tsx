import type { HelperScope } from "../services/response.generated";
import { bytes } from "../stores/model";
const text = (value: string) =>
  bytes(Array.from(new TextEncoder().encode(value)));
export function ReviewedHttpsAuthentication({
  scope,
}: {
  scope?: HelperScope | null;
}) {
  if (!scope) return null;
  return (
    <p>
      Reviewed HTTPS account: {text(scope.account)} · {text(scope.host)}/
      {text(scope.path)}. A recorded review does not grant another credential
      lookup.
    </p>
  );
}
