/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npx wrangler dev src/index.js` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npx wrangler publish src/index.js --name my-worker` to publish your worker
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

async function submitJob(geneList, taxId) {
	const jobUrl = "https://rest.uniprot.org/idmapping/run";
	let formData = new FormData();
	formData.append("from", "Gene_Name");
	formData.append("to", "UniProtKB-Swiss-Prot");
	formData.append("ids", geneList.join(","));
	formData.append("taxId", taxId);
	for (const [key, value] of formData.entries()) {
		console.log(`${key}=${value}`);
	}
	const init = {
		method: "POST",
		body: formData
	};
	const resp = await fetch(jobUrl, init);
	const result = await resp.json();
	console.log(result);
	const jobId = result.jobId;
	return jobId;
}

async function getJobResult(jobId) {
	const jobStatusUrl = `https://rest.uniprot.org/idmapping/status/${jobId}`;
	while (true) {
		const statusResp = await fetch(jobStatusUrl, { redirect: "manual" });
		const status = await statusResp.json();
		console.log(status);
		// if result.results is not null
		if (status.jobStatus == "FINISHED") {
			const jobResultUrl = `https://rest.uniprot.org/idmapping/uniprotkb/results/stream/${jobId}?format=json`;
			const resultResp = await fetch(jobResultUrl, {
				headers: {
					"Accept-Language": "en-US,en;q=0.9",
				}
			});
			console.log(`response headers: ${JSON.stringify(Object.fromEntries(resultResp.headers))}}`);
			const resultText = await resultResp.text();
			console.log(`resultText: ${resultText}`);
			const b64ResultText = btoa((resultText));
			console.log(`${b64ResultText}`);
			const resultJson = JSON.parse(resultText);
			return resultJson.results
		}
		// sleep 1s
		await new Promise(resolve => setTimeout(resolve, 1000));
	}
}



function extractCommentsFromResult(result) {
	const itemObj = result.map(item => {
		let itemObj = {};
		itemObj.gene = item.from;
		itemObj.uniprotId = item.to.uniProtkbId;
		itemObj.uniprotAccession = item.to.primaryAccession;
		itemObj.commentText = item.to.comments.map(comment => {
			let commentText = "";
			if (comment.commentType == "FUNCTION") {
				commentText += comment.texts.map(text => {
					return text.value;
				}).join("\n")
				commentText += "\n";
			}
			return commentText;
		}).filter(singleCommentText => singleCommentText.length > 0).join("\n") + "\n\n";
		return itemObj;
	});
	return itemObj;
}

export default {
	async fetch(request) {
		const pathname = new URL(request.url).pathname;
		if (!pathname.startsWith('/api/v1')) {
			return new Response("Current only support /api/v1/*", { status: 404 });
		}
		const requestBody = await request.json();
		const jobId = await submitJob(requestBody.geneList, requestBody.taxId);
		const idMappingResult = await getJobResult(jobId);
		return new Response(JSON.stringify(extractCommentsFromResult(idMappingResult)), { status: 200 });
	},
};
