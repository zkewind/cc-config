import OpenAI from 'openai';

const openai = new OpenAI({
    apiKey: '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',
    baseURL: 'https://integrate.api.nvidia.com/v1',
})


async function main() {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const completion = await (openai.chat.completions.create as any)({
        model: "deepseek-ai/deepseek-v4-flash",
        messages: [{"role":"user","content":"你好"}],
        temperature: 1,
        top_p: 0.95,
        max_tokens: 16384,
        chat_template_kwargs: {"thinking":true,"reasoning_effort":"high"},
        stream: true
    })

    for await (const chunk of completion) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const delta = chunk.choices[0]?.delta as any;
        const reasoning = delta?.reasoning || delta?.reasoning_content;
        if (reasoning) process.stdout.write(reasoning);
        process.stdout.write(delta?.content || '')

    }

}

main();

