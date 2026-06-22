/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — page data
   bodhi-models-data.js  (plain script, load before the React app)
   Exposes window.MODELS_DATA = { MY_MODELS, LOCAL_MODELS, API_PROVIDERS,
   TAG_MAP, STATUS_CFG, PROV_COLORS }
═══════════════════════════════════════════════════════════════ */
window.MODELS_DATA = (function () {

  const MY_MODELS = [
    { id:'lf1', type:'local-file', org:'afrideva', repo:'Llama-68M-Chat',
      filename:'llama-68m-chat-v1.q8_0.gguf', size:'0.07 GB',
      detail:{ repo:'afrideva/Llama-68M-Chat', filename:'llama-68m-chat-v1.q8_0.gguf', snapshot:'a3f8d2c1e9b4f72a',
        note:'Auto-discovered from local cache. Alias is derived as org/repo:quant and is read-only.' } },
    { id:'lf2', type:'local-file', org:'Qwen', repo:'Qwen3-Coder-32B',
      filename:'Qwen3-Coder-32B-Q4_K_M.gguf', size:'18.5 GB',
      detail:{ repo:'Qwen/Qwen3-Coder-32B', filename:'Qwen3-Coder-32B-Q4_K_M.gguf', snapshot:'b9d4f1a2c3e5d8f0',
        note:'Downloaded via Bodhi. Pull a different quant to replace.' } },
    { id:'ma1', type:'model-alias', org:'', repo:'my-qwen-coder',
      filename:'Qwen/Qwen3-Coder-32B:Q4_K_M', size:'18.5 GB',
      detail:{ repo:'Qwen/Qwen3-Coder-32B', filename:'Qwen3-Coder-32B-Q4_K_M.gguf', snapshot:'b9d4f1a2c3e5d8f0',
        note:'User-created alias with custom system prompt and parameters.' } },
    { id:'lf3', type:'local-file', org:'meta-llama', repo:'Llama-3.3-70B',
      filename:'Llama-3.3-70B-Instruct.Q4_K_M.gguf', size:'35.0 GB',
      detail:{ repo:'meta-llama/Llama-3.3-70B', filename:'Llama-3.3-70B-Instruct.Q4_K_M.gguf', snapshot:'f7d5db77208a',
        note:'Downloaded via Bodhi.' } },
    { id:'am1', type:'api-model', name:'01kp50czqbcgnhnwtnv7jq2s',
      baseUrl:'https://api.anthropic.com/v1', provider:'ANTHROPIC', modelsExposed:1, keyStatus:'no-key',
      detail:{ baseUrl:'https://api.anthropic.com/v1', provider:'ANTHROPIC', models:['claude-sonnet-4-5'] } },
    { id:'am2', type:'api-model', name:'01kp506g2crx8pgqtp4ts1jfh7',
      baseUrl:'https://api.anthropic.com/v1', provider:'ANTHROPIC_OAUTH', modelsExposed:1, keyStatus:'no-key',
      detail:{ baseUrl:'https://api.anthropic.com/v1', provider:'ANTHROPIC_OAUTH', models:['claude-opus-4'] } },
    { id:'am3', type:'api-model', name:'openai-gpt-main',
      baseUrl:'https://api.openai.com/v1', provider:'OPENAI', modelsExposed:3, keyStatus:'connected',
      detail:{ baseUrl:'https://api.openai.com/v1', provider:'OPENAI', models:['gpt-5','gpt-4o','gpt-4o-mini'] } },
    { id:'fb1', type:'fallback', name:'smart-fallback',
      steps:[
        { aliasName:'openai-gpt-main',          aliasType:'api-model',   provider:'OPENAI',    model:'gpt-4o',            enabled:true  },
        { aliasName:'01kp50czqbcgnhnwtnv7jq2s', aliasType:'api-model',   provider:'ANTHROPIC', model:'claude-sonnet-4-5', enabled:false },
        { aliasName:'my-qwen-coder',            aliasType:'model-alias', model:null,                                     enabled:true  },
      ],
      detail:{ note:'Tries OpenAI GPT-4o first. The Anthropic step is temporarily disabled — on error, falls through directly to local Qwen3-Coder.' } },
    { id:'fb2', type:'fallback', name:'local-first',
      steps:[
        { aliasName:'my-qwen-coder',  aliasType:'model-alias', model:null,                                      enabled:true },
        { aliasName:'openai-gpt-main', aliasType:'api-model', provider:'OPENAI', model:'gpt-4o-mini',            enabled:true },
      ],
      detail:{ note:'Prefers local inference for cost and privacy. On error, falls back to OpenAI GPT-4o mini.' } },
  ];

  /* Local (HuggingFace GGUF) models.
     Per-model editorial / catalog fields (Ⓗ HF + Ⓒ curation):
       owner_verified, staff_pick, params, arch, domain, format, license,
       score (Human-eval), dlNum/dlLabel (downloads), likeNum/likeLabel (likes),
       updated (ISO → relative), ctx (context window), maxGB (largest quant).
     quants[].rec marks the recommended default. Per-quant bit-width + host
     fit (Ⓛ BodhiApp-local) are DERIVED in the app from the quant name + size. */
  const LOCAL_MODELS = [
    { rank:1, org:'Qwen', repo:'Qwen3-Coder-32B',
      params:'32B', arch:'Qwen3-MoE', domain:'llm', format:'GGUF', license:'Apache-2.0',
      task:'text-generation', created:'2025-08-20', trending:96, langs:['en','zh'],
      owner_verified:true, staff_pick:true,
      score:74.2, dlNum:443000, dlLabel:'443k', likeNum:9100, likeLabel:'9.1k',
      updated:'2025-09-08', ctx:131072, maxGB:32.1,
      meta:'32B · 5 quants · up to 32.1 GB · Apache-2.0',
      tags:['coding','tool-use','reasoning'], quants:5,
      detail:{
        caps:['tool-use','coding','reasoning','structured-output'],
        specs:[{k:'Context',v:'131,072 tokens'},{k:'Architecture',v:'Qwen3-MoE'},{k:'Parameters',v:'32B'},{k:'License',v:'Apache-2.0'}],
        quants:[{name:'Q8_0',size:'32.1 GB'},{name:'Q6_K',size:'24.6 GB'},{name:'Q4_K_M',size:'18.5 GB',rec:true},{name:'Q3_K_M',size:'14.2 GB'},{name:'Q2_K',size:'10.8 GB'}],
        moreFrom:[{repo:'Qwen3-32B',dl:'1.2M',likes:'45k'},{repo:'Qwen2.5-Coder-14B',dl:'890k',likes:'52k'},{repo:'Qwen3-14B',dl:'620k',likes:'31k'}],
        readme:`# Qwen3-Coder-32B\n\nQwen3-Coder-32B is a **code-specialised** large language model from the Qwen team, tuned for agentic coding, tool use, and long-context repository understanding.\n\n## Highlights\n- Strong code generation, editing, and multi-file reasoning\n- Native **tool / function calling** for agentic workflows\n- **131,072-token** context for whole-repository tasks\n- Apache-2.0 licensed for commercial use\n\n## Recommended build\nPair **Q4_K_M** with a 12-16 GB GPU for the best speed/quality balance. Move up to Q6_K or Q8_0 when you have the VRAM to spare.\n\n## License\nReleased under the **Apache-2.0** license.` } },

    { rank:2, org:'meta-llama', repo:'Llama-3.3-70B',
      params:'70B', arch:'Llama 3.3', domain:'llm', format:'GGUF', license:'Llama',
      task:'text-generation', created:'2025-07-15', trending:71, langs:['en','de','fr','es','it','pt'],
      owner_verified:true, staff_pick:false,
      score:61.4, dlNum:820000, dlLabel:'820k', likeNum:14000, likeLabel:'14k',
      updated:'2025-08-01', ctx:131072, maxGB:70.3,
      meta:'70B · 4 quants · up to 70.3 GB · Llama',
      tags:['reasoning','coding','chat'], quants:4,
      detail:{
        caps:['reasoning','coding','chat'],
        specs:[{k:'Context',v:'131,072 tokens'},{k:'Architecture',v:'Llama 3.3'},{k:'Parameters',v:'70B'},{k:'License',v:'Llama 3.3'}],
        quants:[{name:'Q8_0',size:'70.3 GB'},{name:'Q4_K_M',size:'35.0 GB',rec:true},{name:'Q3_K_M',size:'28.0 GB'},{name:'Q2_K',size:'22.5 GB'}],
        moreFrom:[{repo:'Llama-3.1-8B',dl:'2.4M',likes:'88k'},{repo:'Llama-3.2-3B',dl:'1.1M',likes:'41k'},{repo:'Llama-Guard-3-8B',dl:'180k',likes:'12k'}],
        readme:`# Llama-3.3-70B\n\nMeta's **Llama-3.3-70B-Instruct** is a general-purpose instruction-tuned model with strong reasoning and multilingual ability, delivering near-405B quality at a fraction of the cost.\n\n## Highlights\n- Excellent reasoning, coding, and instruction following\n- **131,072-token** context window\n- Multilingual across 8 core languages\n\n## Memory notes\nThe full 70B is heavy. On a 12 GB GPU you will need **partial CPU offload** even at Q3_K_M. For full-GPU use, prefer a smaller sibling model.\n\n## License\nGoverned by the **Llama 3.3 Community License**.` } },

    { rank:3, org:'deepseek-ai', repo:'DeepSeek-V3',
      params:'671B', arch:'DeepSeek-V3 MoE', domain:'llm', format:'GGUF', license:'DeepSeek',
      task:'text-generation', created:'2025-06-28', trending:64, langs:['en','zh'],
      owner_verified:true, staff_pick:false,
      score:62.7, dlNum:310000, dlLabel:'310k', likeNum:22000, likeLabel:'22k',
      updated:'2025-07-20', ctx:65536, maxGB:120,
      meta:'671B · 3 quants · up to 120 GB · DeepSeek',
      tags:['coding','reasoning'], quants:3,
      detail:{
        caps:['coding','reasoning'],
        specs:[{k:'Context',v:'65,536 tokens'},{k:'Architecture',v:'DeepSeek-V3 MoE'},{k:'Parameters',v:'671B (MoE)'},{k:'License',v:'DeepSeek'}],
        quants:[{name:'Q8_0',size:'120 GB'},{name:'Q4_K_M',size:'60.0 GB',rec:true},{name:'Q2_K',size:'35.0 GB'}],
        moreFrom:[{repo:'DeepSeek-R1',dl:'3.1M',likes:'140k'},{repo:'DeepSeek-Coder-V2',dl:'480k',likes:'56k'}],
        readme:`# DeepSeek-V3\n\n**DeepSeek-V3** is a 671B-parameter Mixture-of-Experts model (37B active per token) with leading performance on coding and reasoning benchmarks.\n\n## Highlights\n- Sparse **MoE** architecture — 671B total, 37B active\n- Strong code and math reasoning\n- **65,536-token** context window\n\n## Memory notes\nEven the **Q2_K** build is 35 GB and will **not** fit a 12 GB GPU. This model is intended for high-memory workstations or servers.\n\n## License\nReleased under the **DeepSeek Model License**.` } },

    { rank:4, org:'google', repo:'gemma-2-9b-it',
      params:'9B', arch:'Gemma 2', domain:'llm', format:'GGUF', license:'Gemma',
      task:'text-generation', created:'2025-05-20', trending:48, langs:['en'],
      owner_verified:true, staff_pick:true,
      score:58.2, dlNum:1200000, dlLabel:'1.2M', likeNum:18000, likeLabel:'18k',
      updated:'2025-06-15', ctx:8192, maxGB:9.4,
      meta:'9B · 4 quants · up to 9.4 GB · Gemma',
      tags:['general','chat'], quants:4,
      detail:{
        caps:['general','chat'],
        specs:[{k:'Context',v:'8,192 tokens'},{k:'Architecture',v:'Gemma 2'},{k:'Parameters',v:'9B'},{k:'License',v:'Gemma'}],
        quants:[{name:'Q8_0',size:'9.4 GB'},{name:'Q6_K',size:'7.8 GB'},{name:'Q4_K_M',size:'5.8 GB',rec:true},{name:'Q2_K',size:'3.8 GB'}],
        moreFrom:[{repo:'gemma-2-27b-it',dl:'620k',likes:'38k'},{repo:'gemma-2-2b-it',dl:'1.4M',likes:'44k'},{repo:'codegemma-7b',dl:'190k',likes:'9k'}],
        readme:`# gemma-2-9b-it\n\nGoogle's **Gemma 2 9B Instruct** is a compact, high-quality open model that punches well above its size class for chat and general tasks.\n\n## Highlights\n- Great quality-per-GB; fully GPU-resident on consumer cards\n- Solid general chat and summarisation\n- **8,192-token** context window\n\n## Recommended build\n**Q4_K_M** (5.8 GB) runs fully on a 12 GB GPU with room for a long context. Q6_K/Q8_0 also fit comfortably.\n\n## License\nGoverned by the **Gemma Terms of Use**.` } },

    { rank:5, org:'microsoft', repo:'Phi-4',
      params:'14B', arch:'Phi-4', domain:'llm', format:'GGUF', license:'MIT',
      task:'text-generation', created:'2025-04-22', trending:42, langs:['en'],
      owner_verified:true, staff_pick:true,
      score:55.1, dlNum:640000, dlLabel:'640k', likeNum:8200, likeLabel:'8.2k',
      updated:'2025-05-10', ctx:16384, maxGB:8.9,
      meta:'14B · 3 quants · up to 8.9 GB · MIT',
      tags:['reasoning','coding'], quants:3,
      detail:{
        caps:['reasoning','coding'],
        specs:[{k:'Context',v:'16,384 tokens'},{k:'Architecture',v:'Phi-4'},{k:'Parameters',v:'14B'},{k:'License',v:'MIT'}],
        quants:[{name:'Q8_0',size:'8.9 GB'},{name:'Q4_K_M',size:'5.1 GB',rec:true},{name:'Q2_K',size:'3.2 GB'}],
        moreFrom:[{repo:'Phi-3.5-mini-instruct',dl:'940k',likes:'33k'},{repo:'Phi-3-medium-128k',dl:'210k',likes:'15k'}],
        readme:`# Phi-4\n\nMicrosoft's **Phi-4** is a 14B model trained heavily on synthetic and curated data, with reasoning and math quality rivalling much larger models.\n\n## Highlights\n- Strong reasoning and math for its size\n- **MIT** licensed — permissive commercial use\n- **16,384-token** context window\n\n## Recommended build\n**Q4_K_M** (5.1 GB) is fully GPU-resident on a 12 GB card.\n\n## License\nReleased under the **MIT** license.` } },

    { rank:6, org:'mistralai', repo:'Mistral-7B-Instruct-v0.3',
      params:'7B', arch:'Mistral', domain:'llm', format:'GGUF', license:'Apache-2.0',
      task:'text-generation', created:'2025-02-10', trending:30, langs:['en','fr','de','es','it'],
      owner_verified:true, staff_pick:false,
      score:49.3, dlNum:2500000, dlLabel:'2.5M', likeNum:31000, likeLabel:'31k',
      updated:'2025-03-12', ctx:32768, maxGB:7.7,
      meta:'7B · 4 quants · up to 7.7 GB · Apache-2.0',
      tags:['chat','multilingual'], quants:4,
      detail:{
        caps:['chat','multilingual'],
        specs:[{k:'Context',v:'32,768 tokens'},{k:'Architecture',v:'Mistral'},{k:'Parameters',v:'7B'},{k:'License',v:'Apache-2.0'}],
        quants:[{name:'Q8_0',size:'7.7 GB'},{name:'Q6_K',size:'6.4 GB'},{name:'Q4_K_M',size:'4.8 GB',rec:true},{name:'Q2_K',size:'3.1 GB'}],
        moreFrom:[{repo:'Mistral-Nemo-Instruct-2407',dl:'1.1M',likes:'47k'},{repo:'Mixtral-8x7B-Instruct',dl:'860k',likes:'72k'},{repo:'Codestral-22B',dl:'290k',likes:'28k'}],
        readme:`# Mistral-7B-Instruct-v0.3\n\n**Mistral-7B-Instruct v0.3** is a fast, popular open model with an updated tokenizer and function-calling support — a dependable everyday workhorse.\n\n## Highlights\n- Very fast; fully GPU-resident on modest hardware\n- Function calling + extended vocabulary\n- **32,768-token** context window\n\n## Recommended build\n**Q4_K_M** (4.8 GB) leaves ample VRAM for long context on a 12 GB GPU.\n\n## License\nReleased under the **Apache-2.0** license.` } },

    { rank:7, org:'Qwen', repo:'Qwen2.5-VL-7B-Instruct',
      params:'7B', arch:'Qwen2.5-VL', domain:'vlm', format:'GGUF', license:'Apache-2.0',
      task:'image-text-to-text', created:'2025-09-01', trending:88, langs:['en','zh'],
      owner_verified:true, staff_pick:true,
      score:0, dlNum:380000, dlLabel:'380k', likeNum:7500, likeLabel:'7.5k',
      updated:'2025-09-25', ctx:128000, maxGB:8.1,
      meta:'7B · 4 quants · up to 8.1 GB · Apache-2.0',
      tags:['vision','tool-use','chat'], quants:4,
      detail:{
        caps:['vision','tool-use','chat','structured-output'],
        specs:[{k:'Context',v:'128,000 tokens'},{k:'Architecture',v:'Qwen2.5-VL'},{k:'Parameters',v:'7B'},{k:'Modality',v:'Image + Text → Text'},{k:'License',v:'Apache-2.0'}],
        quants:[{name:'Q8_0',size:'8.1 GB'},{name:'Q6_K',size:'6.6 GB'},{name:'Q4_K_M',size:'4.9 GB',rec:true},{name:'Q3_K_M',size:'3.9 GB'}],
        moreFrom:[{repo:'Qwen3-Coder-32B',dl:'443k',likes:'9.1k'},{repo:'Qwen2.5-VL-3B-Instruct',dl:'210k',likes:'4.2k'},{repo:'Qwen2-VL-7B',dl:'520k',likes:'18k'}],
        readme:`# Qwen2.5-VL-7B-Instruct\n\n**Qwen2.5-VL** is a multimodal model that accepts **images and text** and returns text — covering document understanding, chart/diagram reasoning, OCR, and visual grounding. Runs locally as GGUF in BodhiApp.\n\n## Highlights\n- **Image-Text-to-Text**: vision question answering, OCR, screenshots, charts\n- Native **tool / function calling** for agentic visual workflows\n- **128,000-token** context for long multimodal documents\n- Apache-2.0 licensed for commercial use\n\n## Recommended build\n**Q4_K_M** (4.9 GB) fits fully on a 12 GB GPU including the vision encoder.\n\n## License\nReleased under the **Apache-2.0** license.` } },

    { rank:8, org:'unsloth', repo:'gemma-3-12b-it',
      params:'12B', arch:'Gemma 3', domain:'vlm', format:'GGUF', license:'Gemma',
      task:'image-text-to-text', created:'2025-09-10', trending:79, langs:['en','es','fr','de','ja','ko'],
      owner_verified:true, staff_pick:false,
      score:0, dlNum:510000, dlLabel:'510k', likeNum:6100, likeLabel:'6.1k',
      updated:'2025-10-02', ctx:131072, maxGB:12.5,
      meta:'12B · 4 quants · up to 12.5 GB · Gemma',
      tags:['vision','chat','multilingual'], quants:4,
      detail:{
        caps:['vision','chat','multilingual'],
        specs:[{k:'Context',v:'131,072 tokens'},{k:'Architecture',v:'Gemma 3'},{k:'Parameters',v:'12B'},{k:'Modality',v:'Image + Text → Text'},{k:'License',v:'Gemma'}],
        quants:[{name:'Q8_0',size:'12.5 GB'},{name:'Q6_K',size:'10.1 GB'},{name:'Q4_K_M',size:'7.3 GB',rec:true},{name:'Q3_K_M',size:'5.9 GB'}],
        moreFrom:[{repo:'gemma-3-4b-it',dl:'880k',likes:'21k'},{repo:'gemma-3-27b-it',dl:'340k',likes:'15k'},{repo:'gemma-2-9b-it',dl:'1.2M',likes:'18k'}],
        readme:`# gemma-3-12b-it\n\n**Gemma 3 12B** is Google's open multimodal model, accepting **interleaved images and text**. This GGUF build (by unsloth) runs locally in BodhiApp.\n\n## Highlights\n- **Image-Text-to-Text** vision understanding across 35+ languages\n- **131,072-token** context window\n- Strong everyday chat and summarisation quality-per-GB\n\n## Memory notes\n**Q4_K_M** (7.3 GB) runs fully on a 12 GB GPU. The vision tower adds a little overhead — leave ~1 GB headroom.\n\n## License\nGoverned by the **Gemma Terms of Use**.` } },
  ];

  /* API providers. Per-model rows carry caps / context / pricing ($ per
     1M tokens, input / output) shown in the detail rail's model table.
     `connected` + `apiModels` are DERIVED below from MY_MODELS — a provider
     reads as Connected only when an API model of its type exists. */
  const API_PROVIDERS = [
    { rank:1, provider:'Anthropic', slug:'anthropic', format:'Anthropic',
      tags:['tool-use','reasoning','vision'], models:8,
      modelRows:[
        { name:'claude-sonnet-4-5', caps:['tool-use','reasoning','vision'], ctx:'200K', in:3,    out:15 },
        { name:'claude-opus-4',     caps:['tool-use','reasoning','vision'], ctx:'200K', in:15,   out:75 },
        { name:'claude-haiku-3-5',  caps:['tool-use','vision'],             ctx:'200K', in:0.80, out:4 },
        { name:'claude-3-5-sonnet', caps:['tool-use','reasoning','vision'], ctx:'200K', in:3,    out:15 },
        { name:'claude-3-5-haiku',  caps:['tool-use'],                      ctx:'200K', in:0.80, out:4 },
        { name:'claude-3-opus',     caps:['reasoning','vision'],            ctx:'200K', in:15,   out:75 },
        { name:'claude-3-sonnet',   caps:['vision'],                        ctx:'200K', in:3,    out:15 },
        { name:'claude-3-haiku',    caps:['chat'],                          ctx:'200K', in:0.25, out:1.25 },
      ] },
    { rank:2, provider:'OpenAI', slug:'openai', format:'OpenAI',
      tags:['tool-use','vision','reasoning'], models:12,
      modelRows:[
        { name:'gpt-5',       caps:['tool-use','reasoning','vision'], ctx:'256K', in:1.25, out:10 },
        { name:'gpt-4o',      caps:['tool-use','vision'],             ctx:'128K', in:2.50, out:10 },
        { name:'gpt-4o-mini', caps:['tool-use','vision'],             ctx:'128K', in:0.15, out:0.60 },
        { name:'gpt-4-turbo', caps:['tool-use','vision'],             ctx:'128K', in:10,   out:30 },
        { name:'o3',          caps:['reasoning','tool-use'],          ctx:'200K', in:10,   out:40 },
        { name:'o3-mini',     caps:['reasoning'],                     ctx:'200K', in:1.10, out:4.40 },
        { name:'o4-mini',     caps:['reasoning','vision'],            ctx:'200K', in:1.10, out:4.40 },
      ] },
    { rank:3, provider:'OpenRouter', slug:'openrouter', format:'OpenAI',
      tags:['tool-use','reasoning','vision'], models:200,
      modelRows:[
        { name:'meta-llama/llama-3.3-70b',      caps:['tool-use','reasoning'],        ctx:'131K', in:0.12, out:0.30 },
        { name:'google/gemini-2.0-flash',       caps:['tool-use','vision'],           ctx:'1M',   in:0.10, out:0.40 },
        { name:'anthropic/claude-sonnet-4-5',   caps:['tool-use','reasoning','vision'],ctx:'200K', in:3,    out:15 },
        { name:'deepseek/deepseek-v3',          caps:['reasoning','coding'],          ctx:'64K',  in:0.27, out:1.10 },
      ] },
    { rank:4, provider:'Groq', slug:'groq', format:'OpenAI',
      tags:['reasoning','multilingual'], models:15,
      modelRows:[
        { name:'llama-3.3-70b-versatile', caps:['tool-use','reasoning'], ctx:'128K', in:0.59, out:0.79 },
        { name:'mixtral-8x7b-32768',      caps:['chat'],                 ctx:'32K',  in:0.24, out:0.24 },
        { name:'gemma2-9b-it',            caps:['chat'],                 ctx:'8K',   in:0.20, out:0.20 },
        { name:'llama-3.1-8b-instant',    caps:['chat'],                 ctx:'128K', in:0.05, out:0.08 },
      ] },
    { rank:5, provider:'NVIDIA NIM', slug:'nvidia-nim', format:'OpenAI',
      tags:['reasoning','vision','coding'], models:50,
      modelRows:[
        { name:'meta/llama-3.3-70b-instruct',     caps:['tool-use','reasoning'], ctx:'128K', in:0, out:0 },
        { name:'nvidia/llama-3.1-nemotron-70b',   caps:['reasoning'],            ctx:'128K', in:0, out:0 },
        { name:'microsoft/phi-4',                 caps:['reasoning','coding'],   ctx:'16K',  in:0, out:0 },
      ] },
    { rank:6, provider:'Together AI', slug:'together', format:'OpenAI',
      tags:['reasoning','coding','vision'], models:100,
      modelRows:[
        { name:'meta-llama/Llama-3.3-70B-Instruct-Turbo', caps:['tool-use','reasoning'], ctx:'131K', in:0.88, out:0.88 },
        { name:'deepseek-ai/DeepSeek-V3',                 caps:['reasoning','coding'],   ctx:'64K',  in:1.25, out:1.25 },
        { name:'Qwen/Qwen3-235B-A22B',                    caps:['reasoning','coding'],   ctx:'128K', in:0.20, out:0.60 },
      ] },
  ];

  /* Derive connection state from My Models: a provider is "Connected" when at
     least one api-model of its type exists. Also collect those models so the
     detail rail can deep-link back to them in My Models. */
  const PROVIDER_OF = { ANTHROPIC:'anthropic', ANTHROPIC_OAUTH:'anthropic', OPENAI:'openai',
    OPENROUTER:'openrouter', GROQ:'groq', NVIDIA:'nvidia-nim', TOGETHER:'together' };
  API_PROVIDERS.forEach((p) => {
    p.apiModels = MY_MODELS.filter((m) => m.type === 'api-model' && PROVIDER_OF[m.provider] === p.slug)
      .map((m) => ({ id: m.id, name: m.name, keyStatus: m.keyStatus }));
    p.connected = p.apiModels.length > 0;
  });

  const TAG_MAP = {
    'tool-use':'tag-indigo','reasoning':'tag-indigo','coding':'tag-leaf',
    'vision':'tag-indigo','structured':'tag-muted','structured-output':'tag-muted',
    'function-calling':'tag-muted','general':'tag-muted','chat':'tag-muted',
    'multilingual':'tag-saffron','embedding':'tag-muted',
  };
  const STATUS_CFG = {
    'connected': { cls:'status-connected', lbl:'Connected',   icon:'check-circle' },
    'api-key':   { cls:'status-apikey',    lbl:'API key set', icon:'key' },
    'available': { cls:'status-available', lbl:'Available',   icon:'circle' },
  };
  const PROV_COLORS = {
    'anthropic':'#D97757','openai':'#10a37f','openrouter':'#7c5cfc',
    'groq':'#f55036','nvidia-nim':'#76b900','together':'#0f62fe',
  };

  /* Ⓛ BodhiApp-local host profile — drives the per-quant fit pill + the
     header "Will it run?" badge. Computed by BodhiApp's backend from the
     real machine; mocked here as a mid-range GPU box. */
  const HOST = { vramGB:12, ramGB:32, label:'NVIDIA RTX 4070 · 12 GB VRAM · 32 GB RAM' };

  /* Publisher autocomplete suggestions (Ⓗ — would come from GET /api/v1/orgs?q=).
     The filter input also accepts free text for orgs not listed here. */
  const ORG_SUGGESTIONS = [
    'Qwen','meta-llama','deepseek-ai','google','microsoft','mistralai',
    'NVIDIA','BAAI','01-ai','bartowski','unsloth','TheBloke','lmstudio-community',
  ];

  return { MY_MODELS, LOCAL_MODELS, API_PROVIDERS, TAG_MAP, STATUS_CFG, PROV_COLORS, HOST, ORG_SUGGESTIONS };
})();
