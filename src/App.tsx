import { useContext, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrent } from "@tauri-apps/api/window";
import { Peers } from "./peers/Peers";
import { Peer } from "./peers/types";

function App() {
	const [message, setMessage] = useState("");
	const [nodeId, setNodeId] = useState("");
	const [peers, setPeers] = useState<Peer[]>([]);
	const [connected, setConnected] = useState(false);

	useEffect(() => {
		invoke<string>("get_node_id").then((id) => setNodeId(id));
	}, []);

	const appWindow = getCurrent();

	async function connect() {
		if (!connected) {
			await invoke("connect_veilid");

			await appWindow.listen<string>("app-message", (event) => {
				setMessage(event.payload);
			});

			await appWindow.listen<string>("peers", (event) => {
				const peers = JSON.parse(event.payload);
				setPeers(peers);
				console.log(peers);
			});

			setConnected(true);
		}
	}

	return (
		<div className="container">
			<h1>Velium</h1>

			<button type="button" onClick={connect} disabled={connected}>
				{connected ? "Connected" : "Connect"}
			</button>

			<p>Node ID: {nodeId}</p>

			<Peers peers={peers} />
		</div>
	);
}

export default App;
