const STORAGE_KEY = "ai-assistant-client-v1";
const LONG_TEXT_THRESHOLD = 12000;

const defaultSettings = {
  baseUrl: "https://api.openai.com/v1",
  apiKey: "",
  model: "gpt-4.1-mini",
  imageBaseUrl: "",
  imageApiKey: "",
  imageModel: "",
  temperature: 0.7,
  systemPrompt: "你是一个可靠、清晰、耐心的中文 AI 助手。",
  imageSize: "1024x1024",
};

const promptTemplates = [
  {
    title: "代码审查",
    body: "请从正确性、边界条件、可维护性和测试覆盖角度审查下面的代码：",
  },
  {
    title: "学习导师",
    body: "请用循序渐进的方式解释下面这个概念，并给一个小例子：",
  },
  {
    title: "产品助手",
    body: "请把下面需求整理成用户故事、核心流程和验收标准：",
  },
  {
    title: "翻译润色",
    body: "请把下面内容翻译成自然、专业的英文，保留原意：",
  },
];

const els = {
  list: document.querySelector("#conversationList"),
  messages: document.querySelector("#messages"),
  title: document.querySelector("#chatTitle"),
  form: document.querySelector("#chatForm"),
  input: document.querySelector("#messageInput"),
  send: document.querySelector("#sendButton"),
  notice: document.querySelector("#notice"),
  hint: document.querySelector("#composerHint"),
  imageInput: document.querySelector("#imageInput"),
  imageSize: document.querySelector("#imageSizeInput"),
  chatMode: document.querySelector("#chatModeButton"),
  imageMode: document.querySelector("#imageModeButton"),
  attach: document.querySelector("#attachButton"),
  attachmentTray: document.querySelector("#attachmentTray"),
  newChat: document.querySelector("#newChatButton"),
  search: document.querySelector("#conversationSearch"),
  settingsButton: document.querySelector("#settingsButton"),
  settingsModal: document.querySelector("#settingsModal"),
  settingsForm: document.querySelector("#settingsForm"),
  shareButton: document.querySelector("#shareButton"),
  shareModal: document.querySelector("#shareModal"),
  shareList: document.querySelector("#shareList"),
  promptButton: document.querySelector("#promptButton"),
  promptModal: document.querySelector("#promptModal"),
  promptList: document.querySelector("#promptList"),
  quickPrompts: document.querySelector("#quickPrompts"),
  mobileMenu: document.querySelector("#mobileMenuButton"),
  clear: document.querySelector("#clearButton"),
  export: document.querySelector("#exportButton"),
  baseUrl: document.querySelector("#baseUrlInput"),
  apiKey: document.querySelector("#apiKeyInput"),
  imageBaseUrl: document.querySelector("#imageBaseUrlInput"),
  imageApiKey: document.querySelector("#imageApiKeyInput"),
  imageModel: document.querySelector("#imageModelInput"),
  pasteKey: document.querySelector("#pasteKeyButton"),
  model: document.querySelector("#modelInput"),
  temperature: document.querySelector("#temperatureInput"),
  systemPrompt: document.querySelector("#systemPromptInput"),
};

let state = loadState();
let abortController = null;
let pendingAssistantContent = "";
let pendingFrame = 0;
let pendingAttachments = [];
let pendingSaveTimer = 0;
let pendingIdleSave = 0;
let mode = "chat";

function loadState() {
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) || "{}");
    const conversations = Array.isArray(saved.conversations) && saved.conversations.length
      ? saved.conversations
      : [createConversation()];
    conversations.forEach((conversation) => {
      conversation.messages = (conversation.messages || []).map(ensureMessageId);
    });
    return {
      settings: { ...defaultSettings, ...(saved.settings || {}) },
      conversations,
      activeId: saved.activeId || conversations[0].id,
    };
  } catch {
    const first = createConversation();
    return {
      settings: { ...defaultSettings },
      conversations: [first],
      activeId: first.id,
    };
  }
}

function saveStateNow() {
  if (pendingSaveTimer) {
    clearTimeout(pendingSaveTimer);
    pendingSaveTimer = 0;
  }
  if (pendingIdleSave && window.cancelIdleCallback) {
    window.cancelIdleCallback(pendingIdleSave);
  }
  pendingIdleSave = 0;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(createPersistableState(state)));
  } catch {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      settings: state.settings,
      conversations: state.conversations.map((conversation) => ({
        ...conversation,
        messages: conversation.messages.map(sanitizeMessageForStorage),
      })),
      activeId: state.activeId,
    }));
  }
}

function saveState() {
  if (pendingSaveTimer) clearTimeout(pendingSaveTimer);
  pendingSaveTimer = setTimeout(() => {
    pendingSaveTimer = 0;
    if (window.requestIdleCallback) {
      pendingIdleSave = window.requestIdleCallback(() => saveStateNow(), { timeout: 1200 });
    } else {
      saveStateNow();
    }
  }, 250);
}

function createConversation() {
  const now = new Date().toISOString();
  return {
    id: randomId(),
    title: "新的对话",
    createdAt: now,
    updatedAt: now,
    messages: [],
  };
}

function ensureMessageId(message) {
  return {
    id: message.id || randomId(),
    ...message,
  };
}

function activeConversation() {
  return state.conversations.find((item) => item.id === state.activeId) || state.conversations[0];
}

function render() {
  renderSettings();
  renderConversationList();
  renderMessages();
  renderPromptList();
  renderShareList();
  renderAttachmentTray();
  els.notice.classList.toggle("show", !state.settings.apiKey);
}

function renderSettings() {
  els.baseUrl.value = state.settings.baseUrl;
  els.apiKey.value = state.settings.apiKey;
  els.model.value = state.settings.model;
  els.imageBaseUrl.value = state.settings.imageBaseUrl || "";
  els.imageApiKey.value = state.settings.imageApiKey || "";
  els.imageModel.value = state.settings.imageModel || "";
  els.temperature.value = state.settings.temperature;
  els.systemPrompt.value = state.settings.systemPrompt;
  els.imageSize.value = state.settings.imageSize || defaultSettings.imageSize;
  updateModeUi();
}

function renderConversationList() {
  const query = els.search.value.trim().toLowerCase();
  els.list.innerHTML = "";
  state.conversations
    .filter((conversation) => conversation.title.toLowerCase().includes(query))
    .sort((a, b) => new Date(b.updatedAt) - new Date(a.updatedAt))
    .forEach((conversation) => {
      const button = document.createElement("button");
      button.className = `conversation-item${conversation.id === state.activeId ? " active" : ""}`;
      button.innerHTML = `<strong>${escapeHtml(conversation.title)}</strong><span>${conversation.messages.length || 0} 条消息 · ${formatTime(conversation.updatedAt)}</span>`;
      button.addEventListener("click", () => {
        state.activeId = conversation.id;
        saveState();
        document.body.classList.remove("sidebar-open");
        render();
      });
      els.list.appendChild(button);
    });
}

function renderMessages() {
  const conversation = activeConversation();
  els.title.textContent = conversation.title;
  els.messages.innerHTML = "";

  if (!conversation.messages.length) {
    els.messages.innerHTML = `
      <div class="empty-state">
        <article>
          <h2>开始一个清爽的 AI 对话</h2>
          <p>支持本地保存、多图识别、长文本 TXT preview、OpenAI-compatible API 和 Windows 独立安装。</p>
        </article>
      </div>
    `;
    return;
  }

  conversation.messages.forEach((message) => {
    if (!message.id) message.id = randomId();
    els.messages.appendChild(createMessageNode(message));
  });
  els.messages.scrollTop = els.messages.scrollHeight;
}

function createMessageNode(message) {
  const node = document.createElement("article");
  node.className = `message ${message.role}${message.error ? " error" : ""}`;
  const avatar = message.role === "user" ? "你" : "AI";
  const avatarNode = document.createElement("div");
  avatarNode.className = "avatar";
  avatarNode.textContent = avatar;

  const bubble = document.createElement("div");
  bubble.className = "bubble";
  if (message.loading) {
    bubble.innerHTML = '<span class="typing"><span></span><span></span><span></span></span>';
  } else {
    bubble.innerHTML = renderMessageBody(message);
  }

  const action = document.createElement("button");
  action.className = "message-delete";
  action.type = "button";
  action.title = "删除这条记录";
  action.setAttribute("aria-label", "删除这条记录");
  action.innerHTML = '<svg viewBox="0 0 24 24"><path d="M4 7h16M10 11v6M14 11v6M6 7l1 14h10l1-14M9 7V4h6v3" /></svg>';
  action.addEventListener("click", () => deleteMessage(message.id));

  node.append(avatarNode, bubble, action);
  return node;
}

function deleteMessage(messageId) {
  const conversation = activeConversation();
  conversation.messages = conversation.messages.filter((message) => message.id !== messageId);
  conversation.updatedAt = new Date().toISOString();
  saveState();
  render();
}

function renderMessageBody(message) {
  const parts = [];
  if (message.longTextPreview) {
    parts.push(renderTextPreview(message.longTextPreview, getDisplayText(message.content)));
  } else if (message.imageResult) {
    parts.push(renderGeneratedImage(message.imageResult));
  } else {
    parts.push(renderMarkdownLite(getDisplayText(message.content)));
  }

  if (Array.isArray(message.attachments) && message.attachments.length) {
    parts.push(renderImageGrid(message.attachments));
  }

  return parts.filter(Boolean).join("");
}

function renderGeneratedImage(image) {
  if (!image?.url || image.url === "undefined") {
    return '<div class="inline-error">图片地址缺失：接口没有返回可显示的图片 URL 或 base64。</div>';
  }
  return `
    <div class="generated-image">
      <img src="${image.url}" alt="${escapeHtml(image.prompt)}" loading="lazy" />
      <div class="generated-image-actions">
        <span>${escapeHtml(image.size || "image")}</span>
        <button type="button" data-save-image="${escapeHtml(image.url)}" data-filename="${escapeHtml(image.filename || `ai-image-${Date.now()}.png`)}">保存图片</button>
      </div>
    </div>
  `;
}

function renderTextPreview(preview, text = "") {
  const href = preview.url || `data:text/plain;charset=utf-8,${encodeURIComponent(text)}`;
  return `
    <div class="txt-preview">
      <div>
        <strong>${escapeHtml(preview.name)}</strong>
        <span>${formatBytes(preview.size)} · TXT preview</span>
      </div>
      <a href="${href}" target="_blank" rel="noreferrer">打开</a>
    </div>
  `;
}

function renderImageGrid(attachments) {
  const images = attachments
    .filter((item) => item.dataUrl)
    .map((item) => `
      <figure class="image-chip">
        <img src="${item.dataUrl}" alt="${escapeHtml(item.name)}" loading="lazy" />
        <figcaption>${escapeHtml(item.name)}</figcaption>
      </figure>
    `)
    .join("");
  if (!images) return "";
  return `<div class="image-grid">${images}</div>`;
}

function getDisplayText(content) {
  if (Array.isArray(content)) {
    const textPart = content.find((part) => part?.type === "text");
    return textPart?.text || "";
  }
  return content || "";
}

function renderPromptList() {
  els.promptList.innerHTML = "";
  promptTemplates.forEach((prompt) => {
    const button = document.createElement("button");
    button.innerHTML = `<strong>${escapeHtml(prompt.title)}</strong><span>${escapeHtml(prompt.body)}</span>`;
    button.addEventListener("click", () => {
      insertPrompt(prompt.body);
      els.promptModal.close();
    });
    els.promptList.appendChild(button);
  });
}

function renderShareList() {
  if (!els.shareList) return;
  const current = activeConversation();
  els.shareList.innerHTML = "";

  const newButton = document.createElement("button");
  newButton.type = "button";
  newButton.innerHTML = "<strong>转接到新对话</strong><span>复制当前上下文，然后在新对话里继续提问</span>";
  newButton.addEventListener("click", () => shareCurrentConversationToNew());
  els.shareList.appendChild(newButton);

  state.conversations
    .filter((conversation) => conversation.id !== current.id)
    .sort((a, b) => new Date(b.updatedAt) - new Date(a.updatedAt))
    .forEach((conversation) => {
      const button = document.createElement("button");
      button.type = "button";
      button.innerHTML = `<strong>${escapeHtml(conversation.title)}</strong><span>${conversation.messages.length || 0} 条消息 · ${formatTime(conversation.updatedAt)}</span>`;
      button.addEventListener("click", () => shareCurrentConversationTo(conversation.id));
      els.shareList.appendChild(button);
    });
}

function shareCurrentConversationToNew() {
  const target = createConversation();
  target.title = `转接：${activeConversation().title}`.slice(0, 32);
  state.conversations.push(target);
  shareCurrentConversationTo(target.id);
}

function shareCurrentConversationTo(targetId) {
  const source = activeConversation();
  const target = state.conversations.find((conversation) => conversation.id === targetId);
  if (!source || !target || !source.messages.length) return;

  const sharedText = buildSharedConversationText(source);
  target.messages.push({
    id: randomId(),
    role: "user",
    content: sharedText,
    longTextPreview: sharedText.length > LONG_TEXT_THRESHOLD ? createLongTextPreview(sharedText) : null,
  });
  target.updatedAt = new Date().toISOString();
  state.activeId = target.id;
  saveState();
  els.shareModal?.close();
  document.body.classList.remove("sidebar-open");
  render();
  els.input.focus();
}

function buildSharedConversationText(conversation) {
  const lines = conversation.messages
    .filter((message) => !message.loading)
    .map((message) => {
      const name = message.role === "assistant" ? "AI" : "用户";
      const text = getDisplayText(message.content) || (message.imageResult ? `[生成图片] ${message.imageResult.prompt}` : "");
      return `${name}: ${text}`;
    })
    .filter(Boolean)
    .join("\n\n");
  return `请基于下面这段已转接的对话上下文继续回答。你可以先理解上下文，再等待我的下一步问题。\n\n${lines}`;
}

function renderAttachmentTray() {
  els.attachmentTray.innerHTML = "";
  els.attachmentTray.hidden = pendingAttachments.length === 0;
  pendingAttachments.forEach((attachment) => {
    const item = document.createElement("div");
    item.className = "pending-image";
    item.innerHTML = `
      <img src="${attachment.dataUrl}" alt="${escapeHtml(attachment.name)}" />
      <span>${escapeHtml(attachment.name)}</span>
      <button type="button" aria-label="移除图片">
        <svg viewBox="0 0 24 24"><path d="M6 6l12 12M18 6 6 18" /></svg>
      </button>
    `;
    item.querySelector("button").addEventListener("click", () => {
      pendingAttachments = pendingAttachments.filter((entry) => entry.id !== attachment.id);
      renderAttachmentTray();
    });
    els.attachmentTray.appendChild(item);
  });
}

function insertPrompt(text) {
  els.input.value = `${text}${els.input.value ? "\n\n" + els.input.value : ""}`;
  autosizeInput();
  els.input.focus();
}

async function sendMessage(content, attachments = []) {
  if (mode === "image") {
    await generateImage(content);
    return;
  }

  const conversation = activeConversation();
  const trimmed = content.trim();
  const fallbackText = attachments.length ? "请识别这些图片内容，并结合我的要求回答。" : "";
  const text = trimmed || fallbackText;
  const userMessage = createUserMessage(text, attachments);

  conversation.messages.push(userMessage);
  conversation.updatedAt = new Date().toISOString();
  if (conversation.title === "新的对话") {
    conversation.title = createTitle(text, attachments);
  }
  saveState();
  render();

  if (!state.settings.apiKey) {
    conversation.messages.push({
      id: randomId(),
      role: "assistant",
      content: "请先在设置里填写 API Key。你的消息已经保存在本地。",
      error: true,
    });
    saveState();
    render();
    return;
  }

  const assistantMessage = { id: randomId(), role: "assistant", content: "", loading: true };
  conversation.messages.push(assistantMessage);
  render();
  setSending(true);

  try {
    abortController = new AbortController();
    await streamCompletion(conversation, assistantMessage, abortController.signal);
  } catch (error) {
    assistantMessage.content = error.name === "AbortError" ? "已停止生成。" : `请求失败：${error.message}`;
    assistantMessage.error = error.name !== "AbortError";
    assistantMessage.loading = false;
    updateLastAssistantBubble(assistantMessage.content);
  } finally {
    assistantMessage.loading = false;
    abortController = null;
    conversation.updatedAt = new Date().toISOString();
    saveState();
    setSending(false);
    if (assistantMessage.error) {
      renderMessages();
    }
    renderConversationList();
    renderShareList();
  }
}

async function generateImage(prompt) {
  const conversation = activeConversation();
  const text = prompt.trim();
  if (!text) return;

  conversation.messages.push({
    id: randomId(),
    role: "user",
    content: text,
  });
  conversation.updatedAt = new Date().toISOString();
  if (conversation.title === "新的对话") {
    conversation.title = `生图：${text.slice(0, 22)}`;
  }
  saveState();
  render();

  if (!imageProviderSettings().apiKey) {
    conversation.messages.push({
      id: randomId(),
      role: "assistant",
      content: "请先在设置里填写 Image API Key，或填写通用 API Key。生图需要接口提供 /images/generations。",
      error: true,
    });
    saveState();
    render();
    return;
  }

  const assistantMessage = {
    id: randomId(),
    role: "assistant",
    content: "",
    loading: true,
  };
  conversation.messages.push(assistantMessage);
  render();
  setSending(true);

  try {
    const size = els.imageSize.value || defaultSettings.imageSize;
    state.settings.imageSize = size;
    saveState();
    const result = normalizeImageGenerationResult(await requestImageGeneration(text, size));
    if (!result.url) {
      throw new Error("接口返回了结果，但没有可显示的图片地址");
    }
    assistantMessage.loading = false;
    assistantMessage.content = `已生成图片：${text}`;
    assistantMessage.imageResult = {
      url: result.url,
      prompt: text,
      size,
      filename: `ai-image-${Date.now()}.png`,
    };
  } catch (error) {
    assistantMessage.content = `生图失败：${formatError(error)}`;
    assistantMessage.error = true;
  } finally {
    assistantMessage.loading = false;
    conversation.updatedAt = new Date().toISOString();
    saveState();
    setSending(false);
    render();
  }
}

async function requestImageGeneration(prompt, size) {
  const imageSettings = imageProviderSettings();
  if (window.__TAURI_INTERNALS__?.invoke) {
    const result = await window.__TAURI_INTERNALS__.invoke("image_generations", {
      request: {
        baseUrl: imageSettings.baseUrl,
        apiKey: imageSettings.apiKey,
        model: imageSettings.model,
        prompt,
        size,
      },
    });
    return result;
  }

  const response = await fetch(`${imageSettings.baseUrl.replace(/\/$/, "")}/images/generations`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${imageSettings.apiKey}`,
    },
    body: JSON.stringify({
      model: imageSettings.model,
      prompt,
      size,
      n: 1,
    }),
  });

  const body = await response.json();
  if (!response.ok) {
    throw new Error(JSON.stringify(body));
  }

  return body;
}

function imageProviderSettings() {
  return {
    baseUrl: (state.settings.imageBaseUrl || state.settings.baseUrl || defaultSettings.baseUrl).trim(),
    apiKey: (state.settings.imageApiKey || state.settings.apiKey || "").trim(),
    model: (state.settings.imageModel || state.settings.model || defaultSettings.model).trim(),
  };
}

function normalizeImageGenerationResult(result) {
  if (!result) return { url: "" };
  if (typeof result === "string") return { url: result };
  if (result.url) return { url: result.url };
  if (result.b64_json) return { url: `data:image/png;base64,${result.b64_json}` };

  const first = Array.isArray(result.data) ? result.data[0] : null;
  if (first?.url) return { url: first.url };
  if (first?.b64_json) return { url: `data:image/png;base64,${first.b64_json}` };
  if (first?.image_url?.url) return { url: first.image_url.url };

  const output = Array.isArray(result.output) ? result.output : [];
  for (const item of output) {
    const content = Array.isArray(item?.content) ? item.content : [];
    for (const part of content) {
      if (part?.image_url?.url) return { url: part.image_url.url };
      if (part?.b64_json) return { url: `data:image/png;base64,${part.b64_json}` };
    }
  }

  const found = findImageString(result);
  return { url: found || "" };
}

function findImageString(value) {
  if (!value) return "";
  if (typeof value === "string") {
    if (value.startsWith("data:image/") || value.startsWith("http://") || value.startsWith("https://")) {
      return value;
    }
    return "";
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      const found = findImageString(item);
      if (found) return found;
    }
    return "";
  }
  if (typeof value === "object") {
    for (const key of ["url", "image", "image_url", "b64_json"]) {
      const item = value[key];
      if (key === "b64_json" && typeof item === "string") return `data:image/png;base64,${item}`;
      const found = findImageString(item);
      if (found) return found;
    }
    for (const item of Object.values(value)) {
      const found = findImageString(item);
      if (found) return found;
    }
  }
  return "";
}

function formatError(error) {
  if (!error) return "未知错误";
  if (typeof error === "string") return error;
  if (error.message) return error.message;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

function createUserMessage(text, attachments) {
  const imageParts = attachments.map((attachment) => ({
    type: "image_url",
    image_url: { url: attachment.dataUrl },
  }));
  const hasImages = imageParts.length > 0;
  const longTextPreview = text.length > LONG_TEXT_THRESHOLD ? createLongTextPreview(text) : null;
  const content = hasImages
    ? [{ type: "text", text }, ...imageParts]
    : text;

  return {
    id: randomId(),
    role: "user",
    content,
    attachments,
    longTextPreview,
  };
}

function createLongTextPreview(text) {
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  return {
    name: `long-message-${Date.now()}.txt`,
    size: blob.size,
    url: `data:text/plain;charset=utf-8,${encodeURIComponent(text)}`,
  };
}

function createTitle(text, attachments) {
  if (text) return text.slice(0, 28);
  return attachments.length ? `图片识别 (${attachments.length})` : "新的对话";
}

async function streamCompletion(conversation, assistantMessage, signal) {
  if (window.AndroidBridge?.chatCompletions) {
    await streamCompletionOnAndroid(conversation, assistantMessage);
    return;
  }

  if (window.__TAURI_INTERNALS__?.invoke) {
    await streamCompletionOnTauri(conversation, assistantMessage);
    return;
  }

  const useLocalProxy = location.protocol.startsWith("http") && ["localhost", "127.0.0.1"].includes(location.hostname);
  const url = useLocalProxy
    ? "/api/chat/completions"
    : `${state.settings.baseUrl.replace(/\/$/, "")}/chat/completions`;
  const messages = buildApiMessages(conversation);

  const response = await fetch(url, {
    method: "POST",
    signal,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${state.settings.apiKey}`,
      "X-Provider-Base-URL": state.settings.baseUrl,
    },
    body: JSON.stringify({
      model: state.settings.model,
      messages,
      temperature: Number(state.settings.temperature) || 0.7,
      stream: true,
    }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `${response.status} ${response.statusText}`);
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  assistantMessage.loading = false;

  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const rawLine of lines) {
      const line = rawLine.trim();
      if (!line.startsWith("data:")) continue;
      const data = line.slice(5).trim();
      if (data === "[DONE]") return;
      try {
        const payload = JSON.parse(data);
        const delta = payload.choices?.[0]?.delta?.content || "";
        if (delta) {
          assistantMessage.content += delta;
          updateLastAssistantBubble(assistantMessage.content);
        }
      } catch {
        // Ignore partial provider-specific stream fragments.
      }
    }
  }
}

async function streamCompletionOnTauri(conversation, assistantMessage) {
  assistantMessage.loading = false;
  const response = await window.__TAURI_INTERNALS__.invoke("chat_completions", {
    request: {
      baseUrl: state.settings.baseUrl,
      apiKey: state.settings.apiKey,
      model: state.settings.model,
      temperature: Number(state.settings.temperature) || 0.7,
      messages: buildApiMessages(conversation),
    },
  });
  assistantMessage.content = response.content;
  updateLastAssistantBubble(assistantMessage.content);
}

function streamCompletionOnAndroid(conversation, assistantMessage) {
  return new Promise((resolve, reject) => {
    const requestId = randomId();

    window.__androidChatCallbacks ||= {};
    window.__androidChatCallbacks[requestId] = {
      onDelta(delta) {
        assistantMessage.loading = false;
        assistantMessage.content += delta;
        updateLastAssistantBubble(assistantMessage.content);
      },
      onDone() {
        delete window.__androidChatCallbacks[requestId];
        resolve();
      },
      onError(message) {
        delete window.__androidChatCallbacks[requestId];
        reject(new Error(message));
      },
    };

    window.AndroidBridge.chatCompletions(JSON.stringify({
      requestId,
      baseUrl: state.settings.baseUrl,
      apiKey: state.settings.apiKey,
      model: state.settings.model,
      temperature: Number(state.settings.temperature) || 0.7,
      messages: buildApiMessages(conversation),
    }));
  });
}

function buildApiMessages(conversation) {
  return [
    { role: "system", content: state.settings.systemPrompt || defaultSettings.systemPrompt },
    ...conversation.messages
      .filter((message) => !message.loading && !message.error)
      .map(({ role, content }) => ({ role, content: stripMissingImageParts(content) })),
  ];
}

function createPersistableState(source) {
  return {
    settings: source.settings,
    conversations: source.conversations.map((conversation) => ({
      ...conversation,
      messages: conversation.messages.map(sanitizeMessageForStorage),
    })),
    activeId: source.activeId,
  };
}

function sanitizeMessageForStorage(message) {
  return {
    ...message,
    content: stripImageData(message.content),
    longTextPreview: message.longTextPreview
      ? {
          name: message.longTextPreview.name,
          size: message.longTextPreview.size,
        }
      : message.longTextPreview,
    attachments: Array.isArray(message.attachments)
      ? message.attachments.map(({ id, name, type, size }) => ({ id, name, type, size }))
      : message.attachments,
  };
}

function stripImageData(content) {
  if (!Array.isArray(content)) return content;
  return content.map((part) => {
    if (part?.type !== "image_url") return part;
    return { type: "text", text: "[图片已用于本次识别，未写入本地存储以避免残留。]" };
  });
}

function stripMissingImageParts(content) {
  if (!Array.isArray(content)) return content;
  const filtered = content.filter((part) => part?.type !== "image_url" || part.image_url?.url);
  return filtered.length ? filtered : "";
}

function updateLastAssistantBubble(content) {
  pendingAssistantContent = content;
  if (pendingFrame) return;
  pendingFrame = requestAnimationFrame(() => {
    pendingFrame = 0;
    const bubbles = els.messages.querySelectorAll(".message.assistant .bubble");
    const bubble = bubbles[bubbles.length - 1];
    if (bubble) {
      bubble.innerHTML = renderMarkdownLite(pendingAssistantContent);
      els.messages.scrollTop = els.messages.scrollHeight;
    }
  });
}

function setSending(isSending) {
  els.send.disabled = isSending;
  els.send.innerHTML = isSending
    ? '<svg viewBox="0 0 24 24"><path d="M6 6h12v12H6z" /></svg>'
    : '<svg viewBox="0 0 24 24"><path d="m22 2-7 20-4-9-9-4 20-7Z" /><path d="M22 2 11 13" /></svg>';
}

function updateModeUi() {
  els.chatMode?.classList.toggle("active", mode === "chat");
  els.imageMode?.classList.toggle("active", mode === "image");
  els.imageSize.hidden = mode !== "image";
  els.attach.hidden = mode !== "chat";
  els.quickPrompts?.classList.toggle("hide", mode === "image");
  els.input.placeholder = mode === "image"
    ? "描述你想生成的图片，Enter 开始生图"
    : "输入消息，Enter 发送，Shift + Enter 换行";
}

function autosizeInput() {
  els.input.style.height = "auto";
  els.input.style.height = `${Math.min(els.input.scrollHeight, 180)}px`;
}

async function readImages(files) {
  const images = Array.from(files).filter((file) => file.type.startsWith("image/"));
  for (const file of images) {
    const dataUrl = await readAsDataUrl(file);
    pendingAttachments.push({
      id: randomId(),
      name: file.name,
      type: file.type,
      size: file.size,
      dataUrl,
    });
  }
  renderAttachmentTray();
}

function readAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error || new Error("图片读取失败"));
    reader.readAsDataURL(file);
  });
}

function renderMarkdownLite(text) {
  return escapeHtml(text)
    .replace(/```([\s\S]*?)```/g, "<pre><code>$1</code></pre>")
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/\n/g, "<br>");
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function formatTime(value) {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

function formatBytes(value) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function randomId() {
  if (crypto?.randomUUID) return crypto.randomUUID();
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

els.form.addEventListener("submit", (event) => {
  event.preventDefault();
  if (abortController) {
    abortController.abort();
    return;
  }
  const content = els.input.value.trim();
  const attachments = pendingAttachments;
  if (!content && !attachments.length) return;
  pendingAttachments = [];
  els.input.value = "";
  autosizeInput();
  renderAttachmentTray();
  sendMessage(content, attachments);
});

window.addEventListener("pagehide", saveStateNow);
window.addEventListener("beforeunload", saveStateNow);

els.attach.addEventListener("click", () => els.imageInput.click());
els.imageInput.addEventListener("change", async () => {
  try {
    await readImages(els.imageInput.files);
  } finally {
    els.imageInput.value = "";
  }
});

els.input.addEventListener("input", autosizeInput);
els.input.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    els.form.requestSubmit();
  }
});

els.newChat.addEventListener("click", () => {
  const conversation = createConversation();
  state.conversations.push(conversation);
  state.activeId = conversation.id;
  saveState();
  document.body.classList.remove("sidebar-open");
  render();
  els.input.focus();
});

els.search.addEventListener("input", renderConversationList);
els.settingsButton.addEventListener("click", () => els.settingsModal.showModal());
els.shareButton?.addEventListener("click", () => {
  renderShareList();
  els.shareModal.showModal();
});
els.promptButton.addEventListener("click", () => els.promptModal.showModal());
els.mobileMenu.addEventListener("click", () => document.body.classList.toggle("sidebar-open"));

els.settingsForm.addEventListener("submit", (event) => {
  event.preventDefault();
  state.settings = {
    baseUrl: els.baseUrl.value.trim() || defaultSettings.baseUrl,
    apiKey: els.apiKey.value.trim(),
    model: els.model.value.trim() || defaultSettings.model,
    imageBaseUrl: els.imageBaseUrl.value.trim(),
    imageApiKey: els.imageApiKey.value.trim(),
    imageModel: els.imageModel.value.trim(),
    temperature: Number(els.temperature.value) || defaultSettings.temperature,
    systemPrompt: els.systemPrompt.value.trim() || defaultSettings.systemPrompt,
    imageSize: els.imageSize.value || defaultSettings.imageSize,
  };
  saveState();
  els.settingsModal.close();
  render();
});

document.querySelectorAll("[data-prompt]").forEach((button) => {
  button.addEventListener("click", () => insertPrompt(button.dataset.prompt));
});

els.chatMode?.addEventListener("click", () => {
  mode = "chat";
  updateModeUi();
  els.input.focus();
});

els.imageMode?.addEventListener("click", () => {
  mode = "image";
  pendingAttachments = [];
  renderAttachmentTray();
  updateModeUi();
  els.input.focus();
});

els.imageSize?.addEventListener("change", () => {
  state.settings.imageSize = els.imageSize.value;
  saveState();
});

function clearLocalConversations() {
  const first = createConversation();
  state.conversations = [first];
  state.activeId = first.id;
  saveState();
  render();
  if (window.AndroidBridge?.toast) {
    window.AndroidBridge.toast("已清空本地会话");
  }
}

window.__clearLocalConversations = clearLocalConversations;

els.clear.addEventListener("click", () => {
  if (window.AndroidBridge?.confirmClear) {
    window.AndroidBridge.confirmClear();
    return;
  }
  if (!confirm("确认清空所有本地会话？")) return;
  clearLocalConversations();
});

els.export.addEventListener("click", () => {
  const filename = `ai-client-conversations-${Date.now()}.json`;
  const payload = JSON.stringify(state.conversations, null, 2);
  if (window.AndroidBridge?.exportConversations) {
    window.AndroidBridge.exportConversations(payload, filename);
    return;
  }
  const blob = new Blob([payload], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 3000);
});

els.pasteKey?.addEventListener("click", async () => {
  try {
    if (window.AndroidBridge?.pasteClipboard) {
      const value = window.AndroidBridge.pasteClipboard();
      if (value) {
        els.apiKey.value = value.trim();
        els.apiKey.focus();
      }
      return;
    }
    const value = await navigator.clipboard.readText();
    els.apiKey.value = value.trim();
    els.apiKey.focus();
  } catch {
    els.apiKey.focus();
  }
});

els.messages.addEventListener("click", async (event) => {
  const button = event.target.closest("[data-save-image]");
  if (!button) return;
  try {
    await saveImageToChosenPath(button.dataset.saveImage, button.dataset.filename || `ai-image-${Date.now()}.png`);
  } catch (error) {
    alert(`保存失败：${error.message}`);
  }
});

async function saveImageToChosenPath(url, filename) {
  const blob = await imageUrlToBlob(url);
  if (window.showSaveFilePicker) {
    const handle = await window.showSaveFilePicker({
      suggestedName: filename,
      types: [{ description: "PNG Image", accept: { "image/png": [".png"] } }],
    });
    const writable = await handle.createWritable();
    await writable.write(blob);
    await writable.close();
    return;
  }

  const objectUrl = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = objectUrl;
  link.download = filename;
  link.click();
  setTimeout(() => URL.revokeObjectURL(objectUrl), 3000);
}

async function imageUrlToBlob(url) {
  if (url.startsWith("data:")) {
    const response = await fetch(url);
    return response.blob();
  }
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
  return response.blob();
}

render();
