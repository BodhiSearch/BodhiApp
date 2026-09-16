# Sources

Primary references cited in this research. Re-verify provider limits/licensing before implementation.

## Self-hosted tunnel providers

- [frp — fatedier/frp](https://github.com/fatedier/frp) (Apache-2.0, Go)
- [frp custom subdomain docs](https://gofrp.org/en/docs/features/http-https/subdomain/)
- [sish — antoniomika/sish](https://github.com/antoniomika/sish) (MIT, Go)
- [sish — selfhost.directory](https://selfhost.directory/project/sish)
- [zrok — openziti/zrok](https://github.com/openziti/zrok) (Apache-2.0, Go/OpenZiti)
- [rathole — rathole-org/rathole](https://github.com/rathole-org/rathole) (Apache-2.0, Rust)
- [chisel — jpillora/chisel](https://github.com/jpillora/chisel) (MIT, Go)
- [inlets-pro — inlets/inlets-pro](https://github.com/inlets/inlets-pro) (EULA/commercial)
- [inletsctl — pkg.go.dev](https://pkg.go.dev/github.com/inlets/inletsctl) (MIT)
- [headscale — juanfont/headscale](https://github.com/juanfont/headscale) (BSD-3, Go)
- [Headscale Funnel feature request #1040](https://github.com/juanfont/headscale/issues/1040)
- [Headscale Serve feature request #1921](https://github.com/juanfont/headscale/issues/1921)
- [awesome-tunneling — anderspitman](https://github.com/anderspitman/awesome-tunneling)

## Unmetered VPS pricing (cost estimates)

- [OVHcloud VPS (US)](https://us.ovhcloud.com/vps/) (confirmed 2026 VPS-1 $4.54 / VPS-2 $8.50)
- [OVHcloud Unmetered VPS](https://www.ovhcloud.com/en/vps/unmetered-vps/)
- [Contabo pricing](https://contabo.com/en-us/pricing/)
- [Contabo VPS (vpsarena)](https://vpsarena.app/providers/contabo)
- [HostHatch pricing](https://hosthatch.com/pricing)
- [IONOS VPS](https://www.ionos.com/servers/vps)
- [BuyVM KVM slices](https://buyvm.net/kvm-dedicated-server-slices/)

## Provider reviews / sentiment

- [Hetzner Cloud review — Better Stack](https://betterstack.com/community/guides/web-servers/hetzner-cloud-review/)
- [netcup Trustpilot](https://www.trustpilot.com/review/netcup.com)
- [netcup review — vpstier](https://vpstier.com/blog/netcup-vps-review-2026/)
- [Contabo review — HostAdvice](https://hostadvice.com/hosting-company/contabo-reviews/)
- [HostHatch Trustpilot](https://www.trustpilot.com/review/hosthatch.com)
- [BuyVM/Frantech Trustpilot](https://www.trustpilot.com/review/buyvm.net)
- [Vultr review — trusthostreview](https://trusthostreview.com/hosting-company/vultr-review)
- [DigitalOcean review — Better Stack](https://betterstack.com/community/guides/web-servers/digitalocean-review/)
- [Linode/Akamai review — Better Stack](https://betterstack.com/community/guides/web-servers/linode-akamai-review/)
- [UpCloud review — cloudpicked](https://www.cloudpicked.com/upcloud-review-en.html)
- [Scaleway review — HostProdigy](https://www.hostprodigy.com/host/scaleway/)
- [RackNerd review — trusthostreview](https://trusthostreview.com/hosting-company/racknerd-review)
- [GreenCloudVPS review — hostingranked](https://www.hostingranked.com/greencloudvps)
- [Kamatera review — HostAdvice](https://hostadvice.com/hosting-company/kamatera-reviews/)
- [OVHcloud review — HostAdvice](https://hostadvice.com/hosting-company/ovh-reviews/)
- [IONOS review — HostProdigy](https://www.hostprodigy.com/host/ionos/)
- [Alwyzon — Trustpilot](https://www.trustpilot.com/review/www.alwyzon.com)

## Provider pricing / bandwidth (three-region model)

- [Hetzner Cloud](https://www.hetzner.com/cloud/) (DE/FI + US + Singapore)
- [Hetzner new shared vCPU plans (20 TB included)](https://www.hetzner.com/pressroom/new-cx-plans/)
- [Hetzner CX22 pricing — vpsfor.dev](https://vpsfor.dev/posts/hetzner-cx22-pricing-2026/)
- [Hetzner June 2026 price changes](https://byteiota.com/hetzner-june-2026-price-shock/)
- [Vultr pricing — vendorbenchmark](https://vendorbenchmark.com/vendors/vultr-pricing)
- [Vultr bandwidth overage rate — docs](https://docs.vultr.com/support/platform/billing/what-is-the-bandwidth-overage-rate)
- [BuyVM KVM slices](https://buyvm.net/kvm-dedicated-server-slices/)
- [BuyVM review (locations: LV/NY/Miami/Luxembourg)](https://vpstier.com/vps/buyvm/)

## frp features / auth / extensibility

- [frp — fatedier/frp](https://github.com/fatedier/frp)
- [frp HTTP/HTTPS vhost routing — DeepWiki](https://deepwiki.com/fatedier/frp/3.3-http-and-https-virtual-host-routing)
- [frp custom subdomain](https://gofrp.org/en/docs/features/http-https/subdomain/)
- [frp custom domains / vhost HTTP example](https://gofrp.org/en/docs/examples/vhost-http/)
- [frp Basic Auth](https://gofrp.org/en/docs/features/http-https/auth/)
- [frp authentication (token/oidc)](https://gofrp.org/en/docs/features/common/authentication/)
- [frp wildcard HTTP services issue #2804](https://github.com/fatedier/frp/issues/2804)
- [frp HTTP/HTTPS vhost routing — DeepWiki](https://deepwiki.com/fatedier/frp/3.3-http-and-https-virtual-host-routing)
- [frp auth bypass advisory (routeByHTTPUser)](https://github.com/advisories/GHSA-pq96-pwvg-vrr9)
- [frp load balancing with multiple frps — issue #3759](https://github.com/fatedier/frp/issues/3759)
- [frp LICENSE (Apache-2.0)](https://github.com/fatedier/frp/blob/dev/LICENSE)

## DNS / geo-routing

- [Cloudflare wildcard DNS records](https://developers.cloudflare.com/dns/manage-dns-records/reference/wildcard-dns-records/)
- [Cloudflare wildcard proxy for everyone](https://blog.cloudflare.com/wildcard-proxy-for-everyone/)
- [Cloudflare Load Balancing geo steering](https://developers.cloudflare.com/load-balancing/understand-basics/traffic-steering/steering-policies/geo-steering/)
- [Cloudflare Load Balancing pricing](https://costbench.com/software/load-balancers/cloudflare-load-balancing/)
- [gdnsd (self-hosted geo-DNS)](https://github.com/gdnsd/gdnsd)
- [GoDNS (DDNS client, not geo-DNS)](https://github.com/TimothyYe/godns)
- [Route 53 vs Constellix](https://costbench.com/compare/aws-route53-vs-constellix/)

## Cloudflare tunnels

- [Quick Tunnels · Cloudflare One docs](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/trycloudflare/)
- [Cloudflare Tunnel · Cloudflare One docs](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/)
- [Tunnels · Cloudflare Sandbox SDK docs](https://developers.cloudflare.com/sandbox/api/tunnels/)
- [Cloudflare Tunnel docs (public apps)](https://developers.cloudflare.com/tunnel/)
- [Quick Tunnels: Anytime, Anywhere · Cloudflare Blog](https://blog.cloudflare.com/quick-tunnels-anytime-anywhere/)
- [Cloudflare Tunnel — try.cloudflare.com](https://try.cloudflare.com/)

## Cloudflare Rust libraries

- [cloudflared crate — crates.io API](https://crates.io/api/v1/crates/cloudflared) (third-party, 0.0.3, abandoned)
- [cloudflared crate — docs.rs](https://docs.rs/cloudflared/latest/cloudflared/)
- [cloudflare crate — crates.io API](https://crates.io/api/v1/crates/cloudflare) (official cloudflare-rs, 0.14.0)
- [cloudflare crate — docs.rs modules](https://docs.rs/cloudflare/latest/cloudflare/)
- [cloudflare::endpoints::cfd_tunnel — docs.rs](https://docs.rs/cloudflare/latest/cloudflare/endpoints/cfd_tunnel/)
- [cloudflare-rs GitHub](https://github.com/cloudflare/cloudflare-rs)

## cloudflared binary & license

- [cloudflared GitHub](https://github.com/cloudflare/cloudflared)
- [cloudflared License · Cloudflare docs](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/downloads/license/)

## Cloudflare policy / terms

- [Cloudflare Service-Specific Terms](https://www.cloudflare.com/service-specific-terms-application-services/)
- [Cloudflare Website and Online Services Terms of Use](https://www.cloudflare.com/policies/terms/)
- [Cloudflare updated ToS blog](https://blog.cloudflare.com/updated-tos/)

## Tailscale tunnels / Funnel

- [Tailscale Funnel · Tailscale Docs](https://tailscale.com/docs/features/tailscale-funnel)
- [tailscale funnel command · Tailscale Docs](https://tailscale.com/docs/reference/tailscale-cli/funnel)
- [Tailscale pricing](https://tailscale.com/pricing)
- [FR: Allow custom domains for Tailscale Funnel #11563](https://github.com/tailscale/tailscale/issues/11563)
- [Tailscale Funnel setup (third-party limitations summary)](https://mylinux.work/guides/tailscale-funnel-setup/)

## Tailscale Rust libraries

- [tailscale-localapi — docs.rs](https://docs.rs/tailscale-localapi)
- [jtdowney/tailscale-localapi GitHub](https://github.com/jtdowney/tailscale-localapi)
- [caius/tsclient GitHub](https://github.com/caius/tsclient)
- [agentsea/tailscale.rs GitHub](https://github.com/agentsea/tailscale.rs)
- [tailscale-client — lib.rs](https://lib.rs/crates/tailscale-client)

## Tailscale license

- [tailscale GitHub](https://github.com/tailscale/tailscale)
- [tailscale LICENSE](https://github.com/tailscale/tailscale/blob/main/LICENSE)

## 9Router

- [9Router GitHub](https://github.com/decolua/9router)
- [Remote Access Tunnels · 9Router DeepWiki](https://deepwiki.com/decolua/9router/10.2-remote-access-tunnels)
