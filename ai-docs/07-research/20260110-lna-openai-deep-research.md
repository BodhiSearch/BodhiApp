# Local Network Access (LNA) – Purpose and Definition

Modern browsers are now **gating web pages' access to a user's private (local) network** behind an explicit permission prompt. This Local Network Access (LNA) feature blocks cross-site requests from the public internet into private IP ranges unless the user allows them. Such restrictions are needed to prevent CSRF-style attacks on home routers or IoT devices (e.g. "drive-by pharming" exploits) and to stop sites from fingerprinting a user's local network. In practice, **any request from a public website to a private or loopback address** is considered a "local network request." Private addresses include the IETF-reserved ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16 from RFC 1918), IPv4 link-local (169.254.0.0/16 RFC 3927), IPv6 unique-local (fc00::/7 RFC 4193), IPv6 link-local (fe80::/10 RFC 4291), IPv4-mapped IPv6 of these, and loopback (127.0.0.0/8, ::1). For example, Chrome 142+ will prompt the user before allowing a site to "look for and connect to any device on your local network".

![Chrome's Local Network Access permission prompt (Chrome 142+)](chrome-lna-prompt.png)
*Figure: Chrome's Local Network Access permission prompt (Chrome 142+), which asks the user to Allow or Block a site from accessing devices on the local network.*

The **Local Network Access specification** (a WICG draft from Jan 2026) spells out this behavior. It explicitly **requires a secure context (HTTPS)** for any local-network requests, and it mandates that a site must first get the user's permission before sending requests into the local network. Once granted, the permission even **relaxes mixed-content rules** for local URLs (since many local services lack valid TLS). In other words, after the user clicks "Allow," Chrome will permit HTTP calls to 192.168.x.x (normally blocked under HTTPS) from that site.

## Browsers & Timeline

**Chrome (and other Chromium browsers)** were first to implement LNA. Starting in **Chrome 138 (mid-2024)** developers could test LNA behavior by enabling `chrome://flags/#local-network-access-check`. The feature **rolled out by default in Chrome 142 (October 2025)**. The Chrome 142 release notes explicitly state that any public→private or intranet→loopback request is now blocked by default behind a user prompt. (Previously, Chrome was working on a CORS-based "Private Network Access" approach, but that was put on hold in favor of this permission prompt.)

**Other Chromium-based browsers** have followed suit. **Edge (Chromium) 143** (roughly Oct/Nov 2025) enforces the same LNA prompt. For example, reports note that Chrome 142+ and Edge 143+ will now show a prompt for any site accessing a local address. In practice, after Chrome 142 the error string in DevTools for a blocked request reads: **"Permission was denied for this request to access the unknown address space,"** indicating LNA intervened. Importantly, this is a **client-side browser restriction** – as one expert notes, "there's no way to control [LNA] from an HTTP server," it is purely a browser security measure.

**Firefox** has begun implementing a similar feature. Nightly builds (from v143) now block local-network requests by default unless the user grants permission. (Stable Firefox is expected to follow this behavior in a future release.) By contrast, **Safari/WebKit currently has no dedicated web-permission prompt** for local network requests. Safari on iOS/macOS does enforce an OS-level "Local Network Privacy" for native apps (iOS 14+), but it does not yet prompt for local-IP HTTP requests from web pages. (Discussions are ongoing in WebKit bugs about allowing bypasses for localhost, but as of late 2025 no web-wide LNA feature is shipped.)

## How Local Network Access Works (Spec & CORS)

Under the LNA rules, when a site running over HTTPS tries to reach a private IP, the browser triggers the permission flow. If the site has not been allowed, Chrome (and soon Firefox) will show a prompt instead of even sending the request. If the user allows access, the request is sent with normal CORS rules. If the user denies (or ignores) it, the browser blocks the request entirely. Note that prior to Chrome 142, Chrome was experimenting with a **Private Network Access (PNA)** spec: this would have injected a CORS preflight with an `Access-Control-Allow-Private-Network` header requirement.

In fact, servers on the local network were instructed to handle an OPTIONS preflight and respond with `Access-Control-Allow-Private-Network: true` to opt in. (For example, Chrome's PNA blog shows that a private-resource preflight must include `Access-Control-Allow-Private-Network: true` for the fetch to succeed.) **In today's LNA model, these headers are not strictly required once permission is granted.** However, including them does no harm and maintains compatibility with older pre-permission checks.

In practice, your server must still obey normal CORS. A cross-origin request from `https://your-site.com` to `http://192.168.x.x` will only succeed if the local server responds with the appropriate CORS headers (e.g. `Access-Control-Allow-Origin: https://your-site.com`, etc.). It may also include the now-legacy header `Access-Control-Allow-Private-Network: true` on its preflight/OPTIONS response. (Chrome's earlier guidance was to always respond with this header when allowing a private-network request.) In short, after the user grants permission in the browser, your Node/TS backend should simply behave like any CORS-allowing API.

## Backend (Node/TypeScript) Configuration

To make a local backend LNA-friendly, **enable CORS for your frontend** and (optionally) include the legacy PNA header. A typical setup might include:

- **Serve over HTTPS (secure context).** LNA works only from secure origins. If your site is HTTPS, then requests to a local IP (HTTP) will normally be mixed content – but LNA will relax this after permission. In development, you may use localhost/HTTPS or Chrome's `--ignore-certificate-errors` flag on localhost.

- **Set CORS response headers.** For any local-network endpoint that your site will call, add the standard CORS headers. For example, allow your origin and methods, enable credentials if needed, etc. Importantly, you can also (optionally) add `Access-Control-Allow-Private-Network: true` to preflight responses, which was required under the old PNA flow. This ensures compatibility with Chrome's checks.

- **Trust or prompt for the permission.** On the browser side, Chrome 138–141 required enabling the `#local-network-access-check` flag to test these restrictions. Chrome 142+ and Edge 143 prompt automatically. Your frontend can query the permission via `navigator.permissions.query({name: 'local-network-access'})` (or simply watch for the prompt) and handle the "granted" or "denied" state in code. (If users repeatedly deny, the browser may remember their choice.)

In Node/TypeScript (Express), a middleware might look like this:

```javascript
// Example Express middleware to allow LNA requests
app.use((req, res, next) => {
  // Standard CORS headers
  res.setHeader('Access-Control-Allow-Origin', 'https://yoursite.example.com');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  res.setHeader('Access-Control-Allow-Credentials', 'true');

  // Optional header for Private Network Access (PNA) compatibility
  res.setHeader('Access-Control-Allow-Private-Network', 'true');

  next();
});
```

In this example, the new header `Access-Control-Allow-Private-Network: true` is included for completeness. Under Chrome's current LNA prompt model, **it isn't strictly needed once permission is granted**, but it does not hurt and was explicitly required by the earlier PNA checks. The key is that your server does send an appropriate response to any OPTIONS preflight and to the actual request with the matching `Access-Control-Allow-Origin` (and other CORS headers).

## Summary of Steps

- **Secure origin:** Run your frontend on HTTPS (a "secure context") because LNA only activates there. Chrome will then relax mixed-content blocks for local addresses after the user consents.

- **Configure CORS:** Have your backend respond to CORS preflights and requests. Include `Access-Control-Allow-Origin: <frontend-URL>`, allowed methods/headers, and credentials as needed. Optionally set `Access-Control-Allow-Private-Network: true` on OPTIONS responses to satisfy older Chrome checks.

- **Handle permission:** Educate users or use UI messages to expect the "Allow Local Network" prompt. You can check `navigator.permissions.query({name: 'local-network-access'})` to tell if access is granted/denied/prompt. In managed environments, Chrome enterprise policies (`LocalNetworkAccessAllowedForUrls`) can auto-allow specific domains.

Following these guidelines ensures your backend can be contacted from LNA-enabled browsers once permission is granted. In summary, **LNA is ultimately enforced on the client side**, so there is no magic server flag to override it – the best you can do is to properly implement CORS (including the new header for completeness) and trust the browser prompt.

## Sources

1. [Local Network Access - WICG Specification](https://wicg.github.io/local-network-access/)
2. [Chrome 142 | Release notes | Chrome for Developers](https://developer.chrome.com/release-notes/142)
3. [New permission prompt for Local Network Access | Blog | Chrome for Developers](https://developer.chrome.com/blog/local-network-access)
4. [Firefox Nightly 145.0a1, See All New Features, Updates and Fixes](https://www.firefox.com/en-US/firefox/145.0a1/releasenotes/)
5. [dns - CORS error: "Permission was denied for this request to access the unknown address space" only inside office network (Chrome PNA block?) - Stack Overflow](https://stackoverflow.com/questions/79823001/cors-error-permission-was-denied-for-this-request-to-access-the-unknownaddres)
6. [Private Network Access: introducing preflights | Blog | Chrome for Developers](https://developer.chrome.com/blog/private-network-access-preflight)
