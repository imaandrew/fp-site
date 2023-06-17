export async function getTags(): Promise<string[]> {
  console.log("making request");
  const response = await fetch(
    "https://api.github.com/repos/jcog/fp/git/refs/tags"
  );
  const data = await response.json();
  return data.map((item: { ref: string }) => item.ref.split("/").pop());
}
