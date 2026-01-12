import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Roea AI - Agent Orchestrator',
  description: 'AI agent orchestrator that manages, spawns, and coordinates multiple AI coding agents',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="bg-gray-50 min-h-screen">{children}</body>
    </html>
  );
}
