// Cloudflare Pages 入口：只处理首页 /，按访客选择或浏览器语言跳到对应语言版本；
// 其余请求原样交给静态文件。爬虫一般不带 Accept-Language，仍会收录中文首页。
const PATHS = { "zh-CN": "/", en: "/en/", es: "/es/", fr: "/fr/", de: "/de/", ja: "/ja/", "pt-BR": "/pt/", ru: "/ru/", it: "/it/", ko: "/ko/" };
const PRIMARY = { zh: "zh-CN", en: "en", es: "es", fr: "fr", de: "de", ja: "ja", pt: "pt-BR", ru: "ru", it: "it", ko: "ko" };

/** 按 q 值从高到低取第一个站点支持的语言；都不支持时返回 null（留在中文首页）。 */
export function fromAcceptLanguage(header) {
  const ranked = (header || "")
    .split(",")
    .map((part, i) => {
      const [tag, ...params] = part.trim().split(";");
      const q = params.map((p) => p.trim()).find((p) => p.startsWith("q="));
      return { tag: tag.toLowerCase(), q: q ? Number(q.slice(2)) || 0 : 1, i };
    })
    .filter((x) => x.tag && x.q > 0)
    .sort((a, b) => b.q - a.q || a.i - b.i);
  for (const { tag } of ranked) {
    const lang = PRIMARY[tag.split("-")[0]];
    if (lang) return lang;
  }
  return null;
}

/** 语言菜单点选后写入的 mt_lang cookie 优先于浏览器语言。 */
export function chosenLanguage(cookie) {
  const match = /(?:^|;\s*)mt_lang=([^;]+)/.exec(cookie || "");
  const lang = match && decodeURIComponent(match[1]);
  return lang && lang in PATHS ? lang : null;
}

export function redirectFor(request) {
  const url = new URL(request.url);
  if (request.method !== "GET" || (url.pathname !== "/" && url.pathname !== "/index.html")) return null;
  const lang = chosenLanguage(request.headers.get("Cookie")) || fromAcceptLanguage(request.headers.get("Accept-Language"));
  if (!lang || PATHS[lang] === "/") return null;
  return new Response(null, {
    status: 302,
    headers: { Location: PATHS[lang] + url.search, Vary: "Accept-Language, Cookie", "Cache-Control": "private, no-store" },
  });
}

export default {
  async fetch(request, env) {
    const redirect = redirectFor(request);
    if (redirect) return redirect;
    const response = await env.ASSETS.fetch(request);
    if (new URL(request.url).pathname !== "/") return response;
    // 首页内容随请求头变化，告诉缓存按语言与 cookie 区分
    const headers = new Headers(response.headers);
    headers.set("Vary", "Accept-Language, Cookie");
    return new Response(response.body, { status: response.status, statusText: response.statusText, headers });
  },
};
