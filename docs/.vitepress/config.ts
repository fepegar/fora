import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Fora",
  description:
    "A terminal UI and CLI for managing Azure Machine Learning workspaces",
  lang: "en-US",
  lastUpdated: true,
  cleanUrls: true,
  base: "/fora/",

  head: [
    [
      "link",
      {
        rel: "icon",
        type: "image/png",
        href: "/fora/favicon-96x96.png",
        sizes: "96x96",
      },
    ],
    ["link", { rel: "icon", type: "image/svg+xml", href: "/fora/favicon.svg" }],
    ["link", { rel: "shortcut icon", href: "/fora/favicon.ico" }],
    [
      "link",
      {
        rel: "apple-touch-icon",
        sizes: "180x180",
        href: "/fora/apple-touch-icon.png",
      },
    ],
    ["link", { rel: "manifest", href: "/fora/site.webmanifest" }],
  ],

  themeConfig: {
    logo: "/logo.png",
    nav: [
      { text: "Getting Started", link: "/installation" },
      { text: "CLI Reference", link: "/cli/" },
      { text: "Guides", link: "/example" },
    ],

    sidebar: [
      {
        text: "Getting Started",
        items: [
          { text: "Installation", link: "/installation" },
          { text: "Setup", link: "/setup" },
        ],
      },
      {
        text: "CLI Reference",
        items: [
          { text: "Overview", link: "/cli/" },
          { text: "fora", link: "/cli/tui" },
          { text: "fora init", link: "/cli/init" },
          { text: "fora submit", link: "/cli/submit" },
        ],
      },
      {
        text: "Guides",
        items: [{ text: "Example: Submitting a Job", link: "/example" }],
      },
    ],

    socialLinks: [{ icon: "github", link: "https://github.com/samb-t/fora" }],

    editLink: {
      pattern: "https://github.com/samb-t/fora/edit/main/docs/:path",
    },

    footer: {
      message: "Licensed under the MIT License.",
    },
  },
});
