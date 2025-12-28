import { createHash } from "crypto";
import { basename } from "path";
import * as fs from "fs/promises";
import fetch from "node-fetch";

export type PresignRequest = {
  tenant: string;
  repo: string;
  objects: { sha256: string; size: number }[];
  ttl_secs?: number;
};

export type PresignResponse = {
  object: { sha256: string; size: number };
  put_url: string;
  path: string; // s3://...
}[];

export async function sha256File(path: string) {
  const buf = await fs.readFile(path);
  const h = createHash("sha256");
  h.update(buf);
  return { sha256: h.digest("hex"), size: buf.length };
}

export async function presignObjects(baseUrl: string, body: PresignRequest, sid: string) {
  const res = await fetch(`${baseUrl}/repo/presign`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${sid}`
    },
    body: JSON.stringify(body)
  });
  if (!res.ok) throw new Error(`presign failed: ${res.status} ${await res.text()}`);
  return res.json() as Promise<PresignResponse>;
}

export async function uploadPresigned(url: string, file: string) {
  const data = await fs.readFile(file);
  const res = await fetch(url, { method: "PUT", body: data });
  if (!res.ok) throw new Error(`upload failed: ${res.status} ${await res.text()}`);
}

export async function commitRef(baseUrl: string, args: {
  tenant: string; repo: string; ref: string; old: string; new: string; mode: "ff" | "force";
}, sid: string) {
  const res = await fetch(`${baseUrl}/repo/commit-ref`, {
    method: "POST",
    headers: { "Content-Type": "application/json", "Authorization": `Bearer ${sid}` },
    body: JSON.stringify(args)
  });
  if (!res.ok) throw new Error(`commit-ref failed: ${res.status} ${await res.text()}`);
  return res.json() as Promise<{ status: string; link_hash: string }>;
}

export async function pushDirectory(baseUrl: string, tenant: string, repo: string, refName: string, dir: string, sid: string, mode: "ff"|"force" = "ff") {
  // Hash all files in dir (non-recursive for simplicity; extend as needed)
  const files = (await fs.readdir(dir)).map(f => `${dir}/${f}`);
  const objs = await Promise.all(files.map(sha256File));
  const presigned = await presignObjects(baseUrl, { tenant, repo, objects: objs }, sid);
  // upload in parallel
  await Promise.all(presigned.map((p, i) => uploadPresigned(p.put_url, files[i])));
  // For demo: "new" points to SHA256 of a manifest (concat file hashes)
  const manifest = JSON.stringify({ files: objs }, null, 2);
  const mh = createHash("sha256").update(manifest).digest("hex");
  // old is unknown in demo (client would read last ref via API)
  const old = "0000000000000000000000000000000000000000";
  return commitRef(baseUrl, { tenant, repo, ref: refName, old, new: mh, mode }, sid);
}
