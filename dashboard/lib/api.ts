import { config, authHeaders } from "./config"
import { Container, ContainerMetrics, Metrics } from "./types"

export async function fetchContainers(): Promise<Container[]> {
    const response = await fetch(`${config.apiUrl}/api/containers`, {
        method: "GET",
        headers: authHeaders
    })

    if (!response.ok) 
        throw new Error(`Erreur HTTP: ${response.status}`)

    const data = await response.json()

    return data
}

export async function createContainer(
    name: string,
    image: string,
    args: string[],
    start: boolean
): Promise<Container> {
    const response = await fetch(`${config.apiUrl}/api/containers`, {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({name, image, args, start})
    })

    if (!response.ok) 
        throw new Error(`Erreur HTTP: ${response.status}`)

    const data = await response.json()

    return data
}

export async function deleteContainer(id: string): Promise<void> {
    const response = await fetch(`${config.apiUrl}/api/containers/${id}`, {
        method: "DELETE",
        headers: authHeaders,
    })

    if (!response.ok) 
        throw new Error(`Erreur HTTP: ${response.status}`)

    return
}

export async function fetchMetrics(): Promise<Metrics> {
    const response = await fetch(`${config.apiUrl}/metrics`, {
        method: "GET",
        headers: { "Content-Type": "application/json" }
    })

    if (!response.ok)
        throw new Error(`Erreur HTTP: ${response.status}`)

    const data = await response.json()

    return data
}

export async function fetchContainerMetrics(id: string): Promise<ContainerMetrics> {
    const response = await fetch(`${config.apiUrl}/metrics/${id}`, {
        method: "GET",
        headers: { "Content-Type": "application/json" }
    })

    if (!response.ok) 
        throw new Error(`Erreur HTTP: ${response.status}`)

    const data = await response.json()

    return data
}

export async function fetchFiles(): Promise<string[]> {
    const response = await fetch(`${config.apiUrl}/storage`, {
        method: "GET",
        headers: authHeaders,
    })

    if (!response.ok) 
        throw new Error(`Erreur HTTP: ${response.status}`)

    const data = await response.json()

    return data
}

export async function uploadFile(key: string, content: string): Promise<void> {
    const response = await fetch(`${config.apiUrl}/storage/${key}`, {
        method: "PUT",
        headers: { ...authHeaders, "Content-Type": "text/plain" },
        body: content
    })

    if (!response.ok) 
        throw new Error(`Erreur HTTP: ${response.status}`)

    return
}

export async function deleteFile(key: string): Promise<void> {
    const response = await fetch(`${config.apiUrl}/storage/${key}`, {
        method: "DELETE",
        headers: authHeaders,
    })

    if (!response.ok) 
        throw new Error(`Erreur HTTP: ${response.status}`)

    return
}