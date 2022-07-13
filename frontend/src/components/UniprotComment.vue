<script lang="ts">
import { SimplifiedComment } from "../api/comments";
import { defineComponent } from "vue";
export default defineComponent({
    data() {
        let geneList: string = "";
        let taxId: string = "9606";
        return {
            geneList,
            taxId,
        }
    },
    methods: {
        async getComment(): Promise<SimplifiedComment> {
            const geneListSplited = this.geneList.split("\n");
            const requestUrl = "https://batch_uniprot_backend.tpob.workers.dev/api/v1";
            const comment: SimplifiedComment = await fetch(requestUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    "geneList": geneListSplited,
                    "taxId": this.taxId
                })
            }).then(async response => await response.json());
            return comment;
        }
    },
})
</script>
<template>
    <div id="user-input">
        <textarea v-model="geneList" placeholder="input gene list here"></textarea>
        <button @click="getComment">submit</button>
    </div>
</template>
