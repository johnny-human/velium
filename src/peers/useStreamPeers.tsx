import { useEffect, useState } from "react";

export const useStreamPeers = (stream: any) => {
	const [data, setData] = useState<any>({});

	useEffect(() => {
		setData((state) => {
			const updatedData = { ...state, ...stream };
			return updatedData;
		});
	}, [stream]);

	return data;
};
