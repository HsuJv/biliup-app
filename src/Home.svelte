<script lang="ts">
    // 主页面，负责加载模板、监听拖拽事件，切换模板
    import Sidebar from './Sidebar.svelte';
    import Upload from './Upload.svelte';
    import {attach, progress, speed, template, currentTemplate} from "./store";
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";
    import {createPop} from "./common";
    import {setContext} from "svelte";
    import {writable} from "svelte/store";

    // 加载所有模板数据
    let map;
    invoke('load')
        .then((res) => {
            map = res.streamers;
            for (const streamersKey in map) {
                map[streamersKey].files = [];
                if (map[streamersKey]['videos'].length > 0) {
                    map[streamersKey]['videos'].forEach((value) => {
                        map[streamersKey]['files'].push({
                            filename: value.filename,
                            id: value.filename,
                            title: value.title,
                            desc: value.desc,
                            progress: 100,
                            uploaded: 0,
                            speed_uploaded: 0,
                            speed: 0,
                            totalSize: 0,
                            complete: true,
                            process: false,
                        });
                    })
                }
                map[streamersKey].atomicInt = 0;
            }
            $template = map;
            let key = Object.keys($template)[0];
            $currentTemplate.current = key;
            $currentTemplate.selectedTemplate = $template[key];
            console.log(res);
        }).catch((e) => {
            createPop(e);
            console.log(e);
        });

    // 文件拖拽悬停状态
    let fileHover = writable(false);
    setContext("hover", fileHover);
    progress();
    speed();

    // 监听文件拖拽事件
    listen("tauri://drag-drop", (date: {payload: {paths: string[]}}) => {
        console.log("tauri://file-drop", date);
        let f: {name: string, path: string}[] = [];
        date.payload.paths.forEach((value) => {
            let currentFilename: string | undefined;
            if (value.includes("\\")) {
                currentFilename = value.split("\\").pop();
            } else {
                currentFilename = value.split("/").pop();
            }
            if (!currentFilename) {
                console.error(`unable to extract filename from ${value}`);
                return;
            }
            // console.log('Extracted filename:', currentFilename);
            f.push({ name: currentFilename, path: value });
        });
        attach(f);
        $fileHover = false;
        // setContext("hover", fileHover);
    });
    listen("tauri://drag-over", (date) => {
        console.log("tauri://drag-over", date);
        $fileHover = true;
        // setContext("hover", fileHover);
    });
    listen("tauri://drag-leave", (date) => {
        console.log("tauri://drag-leave", date);
        $fileHover = false;
        // setContext("hover", fileHover);
    });
    let items = [];
    $: items = [...Object.keys($template)];
</script>

<div class="flex items-start">
    <Sidebar items="{items}"/>
    <div class="w-screen h-screen overflow-y-auto overflow-x-hidden">
        <div class="min-h-screen">
            {#key $currentTemplate.current}
                <Upload selected={$currentTemplate.current} selectedTemplate="{$currentTemplate.selectedTemplate}"/>
            {/key}
        </div>
    </div>
</div>

<style>
    @import 'filepond/dist/filepond.css';
    @import 'filepond-plugin-image-preview/dist/filepond-plugin-image-preview.css';
</style>
