import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Fora",
  description:
    "A terminal UI and CLI for managing Azure Machine Learning workspaces",
  lang: "en-US",
  lastUpdated: true,
  cleanUrls: true,
  base: "/fora/",

  themeConfig: {
    // nav: [
    //   { text: "Guide", link: "/installation" },
    //   { text: "CLI Reference", link: "/cli/" },
    // ],

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
