import { useForm } from "react-hook-form";
import type { Server } from "../types";
import { defaultServer } from "../types";

type FormValues = Omit<Server, "id">;

interface Props {
  initial?: Server;
  onSave: (values: FormValues) => void;
  onCancel: () => void;
}

export function ServerForm({ initial, onSave, onCancel }: Props) {
  const { register, handleSubmit, formState: { errors } } = useForm<FormValues>({
    defaultValues: initial ?? { name: "", ...defaultServer() },
  });

  return (
    <form onSubmit={handleSubmit(onSave)} className="space-y-4">
      <h2 className="text-lg font-semibold">
        {initial ? "Edit Server" : "Add Server"}
      </h2>

      {/* Basic fields */}
      <Field label="Name" error={errors.name?.message}>
        <input {...register("name", { required: "Required" })} placeholder="My VPN" />
      </Field>
      <Field label="IP Address" error={errors.ipAddress?.message}>
        <input {...register("ipAddress")} placeholder="1.2.3.4" />
      </Field>
      <Field label="Domain (hostname)" error={errors.domain?.message}>
        <input {...register("domain", { required: "Required" })} placeholder="vpn.example.com" />
      </Field>
      <Field label="Username">
        <input {...register("username")} autoComplete="off" />
      </Field>
      <Field label="Password">
        <input {...register("password")} type="password" autoComplete="off" />
      </Field>

      {/* Protocol */}
      <div className="grid grid-cols-2 gap-3">
        <Field label="Protocol">
          <select {...register("protocol")}>
            <option value="http2">HTTP/2</option>
            <option value="http3">HTTP/3 (QUIC)</option>
          </select>
        </Field>
        <Field label="Fallback Protocol">
          <select {...register("fallbackProtocol")}>
            <option value="">None</option>
            <option value="http2">HTTP/2</option>
            <option value="http3">HTTP/3 (QUIC)</option>
          </select>
        </Field>
      </div>

      {/* Toggles */}
      <div className="grid grid-cols-2 gap-2">
        <Toggle label="Kill Switch" name="killSwitch" register={register} />
        <Toggle label="Post-Quantum" name="postQuantum" register={register} />
        <Toggle label="Anti-DPI" name="antiDpi" register={register} />
        <Toggle label="Skip Cert Verification" name="skipVerification" register={register} />
      </div>

      {/* MTU */}
      <Field label="MTU Size">
        <input
          type="number"
          {...register("mtuSize", { valueAsNumber: true, min: 576, max: 9000 })}
        />
      </Field>

      {/* DNS Upstreams */}
      <Field label="DNS Upstreams (one per line)">
        <textarea
          rows={3}
          {...register("dnsUpstreams", {
            setValueAs: (v: string) =>
              v.split("\n").map((l) => l.trim()).filter(Boolean),
          })}
          defaultValue={initial?.dnsUpstreams.join("\n") ?? ""}
          placeholder="8.8.8.8&#10;tls://1.1.1.1"
        />
      </Field>

      {/* Excluded Routes */}
      <Field label="Excluded Routes (CIDR, one per line)">
        <textarea
          rows={3}
          {...register("excludedRoutes", {
            setValueAs: (v: string) =>
              v.split("\n").map((l) => l.trim()).filter(Boolean),
          })}
          defaultValue={initial?.excludedRoutes.join("\n") ?? ""}
          placeholder="192.168.0.0/16&#10;10.0.0.0/8"
        />
      </Field>

      {/* Certificate */}
      <Field label="Custom Certificate (PEM)">
        <textarea rows={4} {...register("certificate")} placeholder="-----BEGIN CERTIFICATE-----" />
      </Field>

      <div className="flex gap-3 pt-2">
        <button
          type="submit"
          className="flex-1 py-2 rounded-lg bg-blue-600 hover:bg-blue-700 font-medium"
        >
          Save
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="flex-1 py-2 rounded-lg bg-gray-700 hover:bg-gray-600"
        >
          Cancel
        </button>
      </div>
    </form>
  );
}

function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1">
      <label className="text-sm text-gray-300">{label}</label>
      <div className="[&>input]:w-full [&>input]:bg-gray-800 [&>input]:border [&>input]:border-gray-600 [&>input]:rounded-md [&>input]:px-3 [&>input]:py-1.5 [&>input]:text-sm [&>input]:outline-none [&>input]:focus:border-blue-500 [&>select]:w-full [&>select]:bg-gray-800 [&>select]:border [&>select]:border-gray-600 [&>select]:rounded-md [&>select]:px-3 [&>select]:py-1.5 [&>select]:text-sm [&>textarea]:w-full [&>textarea]:bg-gray-800 [&>textarea]:border [&>textarea]:border-gray-600 [&>textarea]:rounded-md [&>textarea]:px-3 [&>textarea]:py-1.5 [&>textarea]:text-sm [&>textarea]:outline-none [&>textarea]:focus:border-blue-500">
        {children}
      </div>
      {error && <p className="text-xs text-red-400">{error}</p>}
    </div>
  );
}

function Toggle({
  label,
  name,
  register,
}: {
  label: string;
  name: keyof Pick<Server, "killSwitch" | "postQuantum" | "antiDpi" | "skipVerification">;
  register: ReturnType<typeof useForm<Omit<Server, "id">>>["register"];
}) {
  return (
    <label className="flex items-center gap-2 cursor-pointer">
      <input type="checkbox" {...register(name)} className="w-4 h-4 accent-blue-500" />
      <span className="text-sm text-gray-300">{label}</span>
    </label>
  );
}
