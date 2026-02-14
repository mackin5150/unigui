import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

  const [importPath, setImportPath] = useState<string>("");
  const [importMsg, setImportMsg] = useState<string>("");

  async function doImport() {
    setImportMsg("");
    const p = importPath.trim();
    if (!p) return;

    try {
      setImportMsg("Importing...");
      await invoke("import_tool", { exePath: p, toolId: null, title: null });
      const updated = await invoke<CommandSpec[]>("list_specs");
      setSpecs(updated);
      setImportMsg("Imported ✅");
    } catch (e: any) {
      setImportMsg(`Import failed: ${String(e)}`);
    }
  }


type CommandSpec = {
  id: string;
  title: string;
  description: string;
  program: string;
  args: { name: string; kind: any; required: boolean; default?: any; help?: string }[];
  io: { stdin: boolean; stdout: boolean; stderr: boolean; outputs_files: boolean };
};

type NodeInvocation = {
  spec_id: string;
  values: Record<string, any>;
  stdin_text?: string | null;
  workdir?: string | null;
};

type RunResult = { stdout: string; stderr: string; exit_code: number };

type NodeModel = {
  nodeId: string;
  specId: string;
  x: number;
  y: number;
  values: Record<string, any>;
  stdin_text?: string;
};

function uid() {
  return Math.random().toString(16).slice(2);
}

export default function App() {
  const [specs, setSpecs] = useState<CommandSpec[]>([]);
  const [nodes, setNodes] = useState<NodeModel[]>([]);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [logs, setLogs] = useState<string>("");

  useEffect(() => {
    invoke<CommandSpec[]>("list_specs").then(setSpecs);
  }, []);

  const selected = useMemo(
    () => nodes.find((n) => n.nodeId === selectedNodeId) || null,
    [nodes, selectedNodeId]
  );

  const selectedSpec = useMemo(() => {
    if (!selected) return null;
    return specs.find((s) => s.id === selected.specId) || null;
  }, [selected, specs]);

  function addNode(specId: string) {
    const node: NodeModel = {
      nodeId: uid(),
      specId,
      x: 40 + nodes.length * 30,
      y: 60 + nodes.length * 20,
      values: {},
      stdin_text: "",
    };
    setNodes((p) => [...p, node]);
    setSelectedNodeId(node.nodeId);
  }

  async function runSelected() {
    if (!selected) return;
    const inv: NodeInvocation = {
      spec_id: selected.specId,
      values: selected.values,
      stdin_text: selected.stdin_text ?? null,
      workdir: null,
    };
    setLogs((p) => p + `\n▶ Running ${selected.specId}\n`);
    const res = await invoke<RunResult>("run_node", { invocation: inv });
    setLogs((p) => p + `exit=${res.exit_code}\n${res.stdout}${res.stderr}\n`);
  }

  return (
    <div style={{ display: "grid", gridTemplateColumns: "320px 1fr 420px", height: "100vh" }}>
      {/* Left: toolbox */}
      <div style={{ borderRight: "1px solid #ddd", padding: 12, overflow: "auto" }}>
        <h3>Toolbox</h3>
        <p style={{ opacity: 0.8 }}>Add nodes (Phase 1: demo specs). Phase 2: import real CLI tools.</p>
	        <div style={{ marginBottom: 12, padding: 10, border: "1px solid #ddd", borderRadius: 10, background: "white" }}>
          <div style={{ fontWeight: 800, marginBottom: 6 }}>Import CLI tool</div>
          <input
            style={{ width: "100%", padding: 8, marginBottom: 8 }}
            placeholder="/usr/bin/ffmpeg  OR  /home/mack/.local/bin/mytool"
            value={importPath}
            onChange={(e) => setImportPath(e.target.value)}
          />
          <button style={{ width: "100%", padding: 10 }} onClick={doImport}>
            Import
          </button>
          {importMsg && <div style={{ marginTop: 8, fontSize: 12, opacity: 0.85 }}>{importMsg}</div>}
          <div style={{ marginTop: 8, fontSize: 12, opacity: 0.7 }}>
            Tip: use <code>which TOOLNAME</code> to find paths.
          </div>
        </div>
        {specs.map((s) => (
          <button
            key={s.id}
            style={{ display: "block", width: "100%", marginBottom: 8, padding: 10 }}
            onClick={() => addNode(s.id)}
          >
            <div style={{ fontWeight: 700 }}>{s.title}</div>
            <div style={{ fontSize: 12, opacity: 0.75 }}>{s.id}</div>
          </button>
        ))}
      </div>

      {/* Middle: canvas */}
      <div style={{ position: "relative", background: "#fafafa" }}>
        <div style={{ padding: 12, borderBottom: "1px solid #eee", display: "flex", gap: 8 }}>
          <button onClick={runSelected} disabled={!selected}>Run selected</button>
          <button onClick={() => setNodes([])}>Clear</button>
        </div>

        <div style={{ position: "relative", width: "100%", height: "calc(100% - 50px)" }}>
          {nodes.map((n) => {
            const spec = specs.find((s) => s.id === n.specId);
            const selectedStyle = n.nodeId === selectedNodeId ? { outline: "2px solid #333" } : {};
            return (
              <div
                key={n.nodeId}
                onClick={() => setSelectedNodeId(n.nodeId)}
                style={{
                  position: "absolute",
                  left: n.x,
                  top: n.y,
                  width: 220,
                  border: "1px solid #ccc",
                  borderRadius: 10,
                  background: "white",
                  padding: 10,
                  boxShadow: "0 2px 10px rgba(0,0,0,0.06)",
                  cursor: "pointer",
                  ...selectedStyle,
                }}
              >
                <div style={{ fontWeight: 800 }}>{spec?.title ?? n.specId}</div>
                <div style={{ fontSize: 12, opacity: 0.7 }}>{n.specId}</div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Right: inspector + logs */}
      <div style={{ borderLeft: "1px solid #ddd", padding: 12, overflow: "auto" }}>
        <h3>Inspector</h3>

        {!selected || !selectedSpec ? (
          <p style={{ opacity: 0.8 }}>Select a node.</p>
        ) : (
          <>
            <div style={{ fontWeight: 800 }}>{selectedSpec.title}</div>
            <div style={{ fontSize: 12, opacity: 0.75, marginBottom: 12 }}>{selectedSpec.description}</div>

            {selectedSpec.args.map((a) => (
              <div key={a.name} style={{ marginBottom: 10 }}>
                <div style={{ fontSize: 12, fontWeight: 700 }}>{a.name}{a.required ? " *" : ""}</div>
                <input
                  style={{ width: "100%", padding: 8 }}
                  placeholder={a.help ?? ""}
                  value={String(selected.values[a.name] ?? "")}
                  onChange={(e) => {
                    const v = e.target.value;
                    setNodes((prev) =>
                      prev.map((node) =>
                        node.nodeId === selected.nodeId
                          ? { ...node, values: { ...node.values, [a.name]: v } }
                          : node
                      )
                    );
                  }}
                />
              </div>
            ))}

            {selectedSpec.io.stdin && (
              <div style={{ marginTop: 12 }}>
                <div style={{ fontSize: 12, fontWeight: 700 }}>stdin</div>
                <textarea
                  style={{ width: "100%", padding: 8, minHeight: 120 }}
                  value={selected.stdin_text ?? ""}
                  onChange={(e) => {
                    const v = e.target.value;
                    setNodes((prev) =>
                      prev.map((node) =>
                        node.nodeId === selected.nodeId ? { ...node, stdin_text: v } : node
                      )
                    );
                  }}
                />
              </div>
            )}
          </>
        )}

        <hr style={{ margin: "16px 0" }} />
        <h3>Run Log</h3>
        <pre style={{ whiteSpace: "pre-wrap", fontSize: 12, background: "#111", color: "#eee", padding: 10, borderRadius: 10 }}>
          {logs || "—"}
        </pre>
      </div>
    </div>
  );
}
