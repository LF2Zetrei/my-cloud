"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { useEffect, useState } from "react"
import { useTheme } from "next-themes"
import { config } from "@/lib/config"

const content = {
    logo: "☁ my-cloud",
    online: "Online",
    offline: "Offline",
    toggleTheme: "Toggle theme",
}

const links = [
    { href: "/",           label: "Dashboard"  },
    { href: "/containers", label: "Containers" },
    { href: "/storage",    label: "Storage"    },
]

function SunIcon() {
    return (
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="5"/>
            <line x1="12" y1="1" x2="12" y2="3"/>
            <line x1="12" y1="21" x2="12" y2="23"/>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
            <line x1="1" y1="12" x2="3" y2="12"/>
            <line x1="21" y1="12" x2="23" y2="12"/>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
        </svg>
    )
}

function MoonIcon() {
    return (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
    )
}

export function Navbar() {
    const pathname = usePathname()
    const { setTheme, resolvedTheme } = useTheme()
    const [online, setOnline] = useState(false)

    useEffect(() => {
        const check = async () => {
            try {
                const res = await fetch(`${config.apiUrl}/health`)
                setOnline(res.ok)
            } catch {
                setOnline(false)
            }
        }
        check()
        const interval = setInterval(check, 10000)
        return () => clearInterval(interval)
    }, [])

    const isDark = resolvedTheme === "dark"
    const toggleTheme = () => setTheme(isDark ? "light" : "dark")

    return (
        <nav className="sticky top-0 z-50 flex items-center justify-between px-6 h-12 border-b border-gray-200/70 dark:border-white/6 bg-white/75 dark:bg-[#0d1117]/80 backdrop-blur-xl">

            <span className="text-sm font-semibold tracking-tight text-gray-900 dark:text-white select-none">
                {content.logo}
            </span>

            <div className="absolute left-1/2 -translate-x-1/2 flex items-center gap-1">
                {links.map((l) => {
                    const active = pathname === l.href
                    return (
                        <Link
                            key={l.href}
                            href={l.href}
                            className={`relative px-3 py-1 rounded-md text-sm transition-colors ${
                                active
                                    ? "text-gray-900 dark:text-white font-medium"
                                    : "text-gray-500 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
                            }`}
                        >
                            {l.label}
                            {active && (
                                <span className="absolute inset-x-1 -bottom-3.25 h-px bg-gray-900 dark:bg-white rounded-full" />
                            )}
                        </Link>
                    )
                })}
            </div>

            <div className="flex items-center gap-3">
                <div className="flex items-center gap-1.5">
                    <span className={`w-1.5 h-1.5 rounded-full ${online ? "bg-emerald-400" : "bg-red-400"}`} />
                    <span className="text-xs text-gray-400 dark:text-gray-500">
                        {online ? content.online : content.offline}
                    </span>
                </div>

                <div className="w-px h-4 bg-gray-200 dark:bg-white/10" />

                <button
                    onClick={toggleTheme}
                    aria-label={content.toggleTheme}
                    suppressHydrationWarning
                    className="flex items-center justify-center w-7 h-7 rounded-md text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/8 transition-colors"
                >
                    {isDark ? <SunIcon /> : <MoonIcon />}
                </button>
            </div>
        </nav>
    )
}
