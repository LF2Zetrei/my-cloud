"use client"

import { useEffect, useRef } from "react"
import { useTheme } from "next-themes"

const content = {
    label: "my-cloud",
    title: "My Cloud",
}

type Props = { subtitle: string }

export function MetaballHero({ subtitle }: Props) {
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const animRef   = useRef<number>(0)
    const { resolvedTheme } = useTheme()

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas) return
        const ctx = canvas.getContext("2d")
        if (!ctx) return

        canvas.width  = canvas.offsetWidth
        canvas.height = canvas.offsetHeight
        const W = canvas.width
        const H = canvas.height

        const isDark = resolvedTheme === "dark"

        const balls = Array.from({ length: 14 }, () => ({
            x:  Math.random() * W,
            y:  Math.random() * H,
            vx: (Math.random() - 0.5) * 1.1,
            vy: (Math.random() - 0.5) * 1.1,
            r:  Math.random() * 55 + 20,
        }))

        const ballColor = isDark ? "rgba(80,130,255,0.14)" : "rgba(99,120,200,0.13)"
        const ballFade  = isDark ? "rgba(80,130,255,0)"    : "rgba(99,120,200,0)"

        const draw = () => {
            ctx.clearRect(0, 0, W, H)
            for (const b of balls) {
                b.x += b.vx
                b.y += b.vy
                if (b.x < 0 || b.x > W) b.vx *= -1
                if (b.y < 0 || b.y > H) b.vy *= -1
                const g = ctx.createRadialGradient(b.x, b.y, 0, b.x, b.y, b.r)
                g.addColorStop(0, ballColor)
                g.addColorStop(1, ballFade)
                ctx.beginPath()
                ctx.arc(b.x, b.y, b.r, 0, Math.PI * 2)
                ctx.fillStyle = g
                ctx.fill()
            }
            animRef.current = requestAnimationFrame(draw)
        }

        draw()
        return () => cancelAnimationFrame(animRef.current)
    }, [resolvedTheme])

    return (
        <div className="relative h-40 overflow-hidden bg-[#e8ecf4] dark:bg-[#07090f] border-b border-gray-200/80 dark:border-white/4">
            <canvas ref={canvasRef} className="absolute inset-0 w-full h-full" />
            <div className="relative z-10 flex flex-col justify-center h-full px-8">
                <p className="text-xs font-medium text-gray-400 dark:text-gray-500 uppercase tracking-widest mb-1.5">
                    {content.label}
                </p>
                <h1 className="text-2xl font-semibold tracking-tight text-gray-900 dark:text-white">
                    {content.title}
                </h1>
                <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    {subtitle}
                </p>
            </div>
        </div>
    )
}
