import type { Metadata } from "next";
import { headers } from "next/headers";
import { IBM_Plex_Mono, Manrope } from "next/font/google";
import "./globals.css";

const manrope = Manrope({
  variable: "--font-manrope",
  subsets: ["latin"],
});

const plexMono = IBM_Plex_Mono({
  variable: "--font-plex-mono",
  subsets: ["latin"],
  weight: ["400", "500", "600"],
});

export async function generateMetadata(): Promise<Metadata> {
  const headerList = await headers();
  const host = headerList.get("x-forwarded-host") ?? headerList.get("host");
  const protocol =
    headerList.get("x-forwarded-proto") ??
    (host?.startsWith("localhost") ? "http" : "https");
  const base = new URL(`${protocol}://${host ?? "localhost:3000"}`);

  return {
    metadataBase: base,
    title: "Zenith — Make Salesforce risk fail at compile time",
    description:
      "A safe, bulk-first language that compiles to readable, deployable Salesforce Apex.",
    openGraph: {
      type: "website",
      title: "Make Salesforce risk fail at compile time.",
      description:
        "A safe, bulk-first language for stronger local guarantees and readable, deployable Apex.",
      siteName: "Zenith",
      images: [
        {
          url: new URL("/og.png", base),
          width: 1200,
          height: 630,
          alt: "Zenith — Make Salesforce risk fail at compile time",
        },
      ],
    },
    twitter: {
      card: "summary_large_image",
      title: "Zenith — compile-time safety for Salesforce",
      description:
        "Shift type, query-shape, security, and governor-limit mistakes into the local feedback loop.",
      images: [new URL("/og.png", base)],
    },
  };
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${manrope.variable} ${plexMono.variable}`}>
        {children}
      </body>
    </html>
  );
}
