import { fetch, Body } from '@tauri-apps/api/http';

import { defaultExtraHeaders, defaultPrompt, defaultRequestArguments, providerPresets } from './Config';

function normalizeRequestPath(requestPath) {
    let path = typeof requestPath === 'string' ? requestPath.trim() : '';
    if (path === '') {
        throw new Error('Request endpoint is required');
    }
    if (!/https?:\/\/.+/.test(path)) {
        path = `https://${path}`;
    }
    const apiUrl = new URL(path);
    if (!apiUrl.pathname.endsWith('/chat/completions')) {
        apiUrl.pathname += apiUrl.pathname.endsWith('/') ? '' : '/';
        apiUrl.pathname += 'chat/completions';
    }
    return apiUrl.href;
}

function parseJsonObject(value, fallback, label) {
    const parsed = JSON.parse(value || fallback);
    if (parsed === null || Array.isArray(parsed) || typeof parsed !== 'object') {
        throw `${label} must be a JSON object`;
    }
    return parsed;
}

function getErrorBody(data) {
    if (typeof data === 'string') {
        return data;
    }
    try {
        return JSON.stringify(data);
    } catch {
        return String(data);
    }
}

function redactLiteral(message, value) {
    if (typeof value !== 'string' || value === '') {
        return message;
    }
    return message.split(value).join('[redacted]');
}

function sanitizeErrorBody(data, redactionValues) {
    let message = getErrorBody(data);
    message = message.replace(/data:image\/[a-z0-9.+-]+;base64,[a-z0-9+/=]+/gi, 'data:image/[redacted]');
    message = message.replace(
        /(["']?(authorization|api-key|x-api-key|apikey|api_key)["']?\s*[:=]\s*["']?)([^"',\n\r}\]]+)/gi,
        '$1[redacted]'
    );
    for (const value of redactionValues) {
        message = redactLiteral(message, value);
    }
    return message;
}

function httpError(status, data, redactionValues) {
    return `Http Request Error\nHttp Status: ${status}\n${sanitizeErrorBody(data, redactionValues)}`;
}

export async function analyze(base64, options) {
    const { config, setResult } = options;
    const {
        requestPath = providerPresets.openai.requestPath,
        model = providerPresets.openai.model,
        apiKey = '',
        authType = 'bearer',
        stream = false,
        prompt = defaultPrompt,
        imageDetail = 'auto',
        requestArguments = defaultRequestArguments,
        extraHeaders = defaultExtraHeaders,
    } = config;

    const selectedModel = typeof model === 'string' ? model.trim() : '';
    if (selectedModel === '') {
        throw new Error('Model is required');
    }

    const parsedExtraHeaders = parseJsonObject(extraHeaders, defaultExtraHeaders, 'Extra headers');
    const headers = {
        'Content-Type': 'application/json',
        ...parsedExtraHeaders,
    };
    if (apiKey) {
        if (authType === 'api_key') {
            headers['api-key'] = apiKey;
        } else {
            headers['Authorization'] = `Bearer ${apiKey}`;
        }
    }
    const redactionValues = [
        apiKey,
        base64,
        ...Object.values(parsedExtraHeaders).filter((value) => typeof value === 'string'),
    ];

    const body = {
        ...parseJsonObject(requestArguments, defaultRequestArguments, 'Request arguments'),
        model: selectedModel,
        stream,
        messages: [
            {
                role: 'user',
                content: [
                    {
                        type: 'text',
                        text: prompt || defaultPrompt,
                    },
                    {
                        type: 'image_url',
                        image_url: {
                            url: `data:image/png;base64,${base64}`,
                            detail: imageDetail || 'auto',
                        },
                    },
                ],
            },
        ],
    };

    const apiUrl = normalizeRequestPath(requestPath);

    if (stream) {
        const res = await window.fetch(apiUrl, {
            method: 'POST',
            headers,
            body: JSON.stringify(body),
        });
        if (!res.ok) {
            throw httpError(res.status, await res.text(), redactionValues);
        }

        let target = '';
        const reader = res.body.getReader();
        try {
            const decoder = new TextDecoder();
            let buffer = '';
            const handleStreamLine = (line) => {
                const trimmed = line.trim();
                if (trimmed === '' || trimmed.startsWith(':') || !trimmed.startsWith('data:')) {
                    return;
                }
                const data = trimmed.slice(5).trim();
                if (data === '' || data === '[DONE]') {
                    return;
                }
                const result = JSON.parse(data);
                const delta = result.choices?.[0]?.delta?.content;
                if (delta) {
                    target += delta;
                    setResult(target);
                }
            };

            while (true) {
                const { done, value } = await reader.read();
                if (done) {
                    if (buffer.trim() !== '') {
                        handleStreamLine(buffer);
                    }
                    setResult(target.trim());
                    return target.trim();
                }
                buffer += decoder.decode(value, { stream: true });
                const lines = buffer.split(/\r?\n/);
                buffer = lines.pop() ?? '';
                for (const line of lines) {
                    handleStreamLine(line);
                }
            }
        } finally {
            reader.releaseLock();
        }
    }

    const res = await fetch(apiUrl, {
        method: 'POST',
        headers,
        body: Body.json(body),
    });
    if (!res.ok) {
        throw httpError(res.status, res.data, redactionValues);
    }
    const content = res.data?.choices?.[0]?.message?.content;
    if (typeof content === 'string' && content.trim() !== '') {
        return content.trim();
    }
    throw sanitizeErrorBody(res.data, redactionValues);
}

export * from './Config';
export * from './info';
