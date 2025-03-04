<script lang="ts">
	import { standardConfig } from '$lib/richText/config/config';
	import { standardTheme } from '$lib/richText/config/theme';
	import { emojiTextNodeTransform } from '$lib/richText/plugins/emoji';
	import {
		$convertToMarkdownString as convertToMarkdownString,
		$convertFromMarkdownString as convertFromMarkdownString
	} from '@lexical/markdown';
	import {
		$createParagraphNode as createParagraphNode,
		$createTextNode as createTextNode,
		$getRoot as getRoot,
		TextNode
	} from 'lexical';
	import { onMount, type Snippet } from 'svelte';
	import {
		Composer,
		ContentEditable,
		RichTextPlugin,
		SharedHistoryPlugin,
		ListPlugin,
		CheckListPlugin,
		AutoFocusPlugin,
		PlaceHolder,
		HashtagPlugin,
		PlainTextPlugin,
		AutoLinkPlugin,
		FloatingLinkEditorPlugin,
		CodeHighlightPlugin,
		CodeActionMenuPlugin,
		MarkdownShortcutPlugin,
		ALL_TRANSFORMERS,
		Toolbar,
		StateStoreRichTextUpdator,
		LinkPlugin
	} from 'svelte-lexical';

	type Props = {
		namespace: string;
		markdown: boolean;
		onError: (error: unknown) => void;
		toolBar?: Snippet;
		plugins?: Snippet;
		placeholder?: string;
	};

	const { namespace, markdown, onError, toolBar, plugins, placeholder }: Props = $props();

	/**
	 * Instance of the lexical composer, used for manipulating the contents of the editor
	 * programatically.
	 */
	let composer = $state<ReturnType<typeof Composer>>();

	/** Standard configuration for our commit message editor. */
	const initialConfig = standardConfig({
		namespace,
		theme: standardTheme,
		onError
	});

	let editorDiv: HTMLDivElement | undefined = $state();

	onMount(() => {
		const unlistenEmoji = composer
			?.getEditor()
			.registerNodeTransform(TextNode, emojiTextNodeTransform);
		return () => {
			unlistenEmoji?.();
		};
	});

	$effect(() => {
		const editor = composer?.getEditor();
		if (markdown) {
			editor?.update(() => {
				convertFromMarkdownString(getRoot().getTextContent(), ALL_TRANSFORMERS);
			});
		} else {
			getPlaintext((text) => {
				editor?.update(() => {
					const root = getRoot();
					root.clear();
					const paragraph = createParagraphNode();
					paragraph.append(createTextNode(text));
					root.append(paragraph);
				});
			});
		}
	});

	export function getPlaintext(callback: (text: string) => void) {
		const editor = composer?.getEditor();
		if (!editor) return;
		const state = editor.getEditorState();
		state.read(() => {
			const markdown = convertToMarkdownString(ALL_TRANSFORMERS);
			callback(markdown);
		});
	}
</script>

<Composer {initialConfig} bind:this={composer}>
	{#if toolBar}
		<Toolbar>
			<StateStoreRichTextUpdator />
			{@render toolBar()}
		</Toolbar>
	{/if}

	<div class="editor-container" bind:this={editorDiv}>
		<div class="editor-scroller">
			<div class="editor">
				<ContentEditable />
				{#if placeholder}
					<PlaceHolder>{placeholder}</PlaceHolder>
				{/if}
			</div>
		</div>

		{#if markdown}
			<AutoFocusPlugin />
			<AutoLinkPlugin />
			<CheckListPlugin />
			<CodeActionMenuPlugin anchorElem={editorDiv} />
			<CodeHighlightPlugin />
			<FloatingLinkEditorPlugin anchorElem={editorDiv} />
			<HashtagPlugin />
			<ListPlugin />
			<LinkPlugin />
			<MarkdownShortcutPlugin transformers={ALL_TRANSFORMERS} />
			<RichTextPlugin />
			<SharedHistoryPlugin />
			{#if plugins}
				{@render plugins()}
			{/if}
		{:else}
			<PlainTextPlugin />
		{/if}
	</div>
</Composer>

<style>
	.editor-container {
		flex-grow: 1;
		background-color: var(--clr-bg-1);
		position: relative;
		display: block;
	}

	.editor-scroller {
		height: 100%;
		/* It's unclear why the resizer is on by default on this element. */
		resize: none;
	}
</style>
