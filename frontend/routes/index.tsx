import type { Handlers, PageProps } from "$fresh/server.ts";

interface Item {
    id: number;
    name: string;
    description: string;
    date_origin: Date;
}

export const handler: Handlers<Item[]> = {
    async GET(_req, ctx) {
        const items: Item[] = await fetch("http://localhost:3000/api/items")
            .then((res) => res.json());
        return ctx.render(items);
    },
};

export default function Items(props: PageProps<Item[]>) {
    return (
        <div>
            <table class="table-auto">
                <thead>
                    <tr>
                        <th class="border px-4 py-2">ID</th>
                        <th class="border px-4 py-2">Name</th>
                        <th class="border px-4 py-2">Description</th>
                        <th class="border px-4 py-2">Date Origin</th>
                    </tr>
                </thead>
                <thead>
                    {props.data.map((item) => (
                        <tr>
                            <td class="border px-4 py-2">{item.id}</td>
                            <td class="border px-4 py-2">{item.name}</td>
                            <td class="border px-4 py-2">{item.description}</td>
                            <td class="border px-4 py-2">{item.date_origin}</td>
                        </tr>
                    ))}
                </thead>
            </table>
        </div>
    );
}
