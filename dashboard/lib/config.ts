export const config = {
    apiUrl: process.env.NEXT_PUBLIC_API_URL ?? "",
    apiToken: process.env.NEXT_PUBLIC_API_TOKEN ?? "",
}

export const authHeaders = {
    "Authorization": `Bearer ${config.apiToken}`,
    "Content-Type": "application/json",
}