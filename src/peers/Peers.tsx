import { memo, useEffect, useState } from "react";
import { Peer } from "./types";
import {
	ColumnDef,
	useReactTable,
	createColumnHelper,
	getCoreRowModel,
	flexRender,
} from "@tanstack/react-table";
import { useStreamPeers } from "./useStreamPeers";

const columnHelper = createColumnHelper<Peer>();

const columns = [
	columnHelper.accessor("node_id", {
		cell: (info) => info.getValue(),
	}),
	columnHelper.accessor("peer_address", {
		cell: (info) => info.getValue(),
	}),
	columnHelper.accessor("time_added", {
		cell: (info) => info.getValue(),
	}),
	columnHelper.accessor("messages_sent", {
		cell: (info) => info.getValue(),
	}),
];

type PeersProps = {
	peers: Peer[];
};

const TableRow = memo(({ rowData, node_id }: any) => {
	console.log(rowData);
	return (
		<tr>
			<td style={{ textAlign: "left" }}>{node_id}</td>
			<td style={{ textAlign: "left" }}>{rowData.peer_address}</td>
			<td style={{ textAlign: "left" }}>{rowData.time_added}</td>
			<td style={{ textAlign: "right" }}>{rowData.messages_sent}</td>
		</tr>
	);
});

export const Peers: React.FC<PeersProps> = (props) => {
	const { peers } = props;
	const data = useStreamPeers(peers);

	const table = useReactTable({
		data,
		columns,
		getCoreRowModel: getCoreRowModel(),
	});

	return (
		<table>
			<thead>
				{table.getHeaderGroups().map((headerGroup) => (
					<tr key={headerGroup.id}>
						{headerGroup.headers.map((header) => (
							<th key={header.id} style={{ textAlign: "left" }}>
								{header.isPlaceholder
									? null
									: flexRender(
											header.column.columnDef.header,
											header.getContext(),
									  )}
							</th>
						))}
					</tr>
				))}
			</thead>
			<tbody>
				{Object.entries(data).map(([key, value]: [string, unknown]) => (
					<TableRow key={key} node_id={key} rowData={value} />
				))}
			</tbody>
		</table>
	);
};
